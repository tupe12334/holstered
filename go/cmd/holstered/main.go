// Command holstered runs the holstered binary from GitHub Releases, so
// `go install github.com/tupe12334/holstered/go/cmd/holstered@latest` works
// even though holstered is written in Rust.
//
// On first run it downloads the release asset for this OS/arch, checks it
// against the release's SHA256SUMS, caches it under the user cache dir, and
// then execs it with the same arguments and stdio.
package main

import (
	"bufio"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"runtime/debug"
	"strings"
)

var releases = "https://github.com/tupe12334/holstered/releases"

func main() {
	bin, err := ensureBinary(version())
	if err != nil {
		fmt.Fprintln(os.Stderr, "holstered:", err)
		os.Exit(1)
	}
	cmd := exec.Command(bin, os.Args[1:]...)
	cmd.Stdin, cmd.Stdout, cmd.Stderr = os.Stdin, os.Stdout, os.Stderr
	if err := cmd.Run(); err != nil {
		var exit *exec.ExitError
		if errors.As(err, &exit) {
			os.Exit(exit.ExitCode())
		}
		fmt.Fprintln(os.Stderr, "holstered:", err)
		os.Exit(1)
	}
}

// version is the module version `go install ...@vX.Y.Z` recorded, which
// matches the binary's release tag. Local builds have none.
func version() string {
	if info, ok := debug.ReadBuildInfo(); ok && strings.HasPrefix(info.Main.Version, "v") {
		return info.Main.Version
	}
	return ""
}

// assetName is the release asset for goos/goarch, as release.yml names it.
func assetName(goos, goarch string) (string, error) {
	switch goos + "/" + goarch {
	case "linux/amd64", "linux/arm64", "darwin/amd64", "darwin/arm64":
		return "holstered-" + goos + "-" + goarch, nil
	case "windows/amd64":
		return "holstered-windows-amd64.exe", nil
	}
	return "", fmt.Errorf("no prebuilt binary for %s/%s; install with `cargo install holstered`", goos, goarch)
}

func ensureBinary(ver string) (string, error) {
	if ver == "" {
		return "", errors.New("unknown version; install with `go install github.com/tupe12334/holstered/go/cmd/holstered@latest`")
	}
	asset, err := assetName(runtime.GOOS, runtime.GOARCH)
	if err != nil {
		return "", err
	}
	cache, err := os.UserCacheDir()
	if err != nil {
		return "", err
	}
	bin := filepath.Join(cache, "holstered", ver, asset)
	if _, err := os.Stat(bin); err == nil {
		return bin, nil
	}

	base := releases + "/download/" + ver + "/"
	sums, err := fetch(base + "SHA256SUMS")
	if err != nil {
		return "", err
	}
	want, err := checksumFor(string(sums), asset)
	if err != nil {
		return "", err
	}
	data, err := fetch(base + asset)
	if err != nil {
		return "", err
	}
	if got := sha256.Sum256(data); hex.EncodeToString(got[:]) != want {
		return "", fmt.Errorf("checksum mismatch for %s %s", asset, ver)
	}

	if err := os.MkdirAll(filepath.Dir(bin), 0o755); err != nil {
		return "", err
	}
	tmp, err := os.CreateTemp(filepath.Dir(bin), asset+".*")
	if err != nil {
		return "", err
	}
	defer os.Remove(tmp.Name())
	if _, err := tmp.Write(data); err != nil {
		tmp.Close()
		return "", err
	}
	if err := tmp.Close(); err != nil {
		return "", err
	}
	if err := os.Chmod(tmp.Name(), 0o755); err != nil {
		return "", err
	}
	return bin, os.Rename(tmp.Name(), bin)
}

// checksumFor finds asset's hash in `sha256sum` output.
func checksumFor(sums, asset string) (string, error) {
	sc := bufio.NewScanner(strings.NewReader(sums))
	for sc.Scan() {
		fields := strings.Fields(sc.Text())
		if len(fields) == 2 && strings.TrimPrefix(fields[1], "*") == asset {
			return fields[0], nil
		}
	}
	return "", fmt.Errorf("%s not listed in SHA256SUMS", asset)
}

func fetch(url string) ([]byte, error) {
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("GET %s: %s", url, resp.Status)
	}
	return io.ReadAll(resp.Body)
}
