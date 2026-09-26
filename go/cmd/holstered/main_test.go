package main

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"runtime"
	"strings"
	"testing"
)

func TestAssetName(t *testing.T) {
	for in, want := range map[[2]string]string{
		{"linux", "amd64"}:   "holstered-linux-amd64",
		{"darwin", "arm64"}:  "holstered-darwin-arm64",
		{"windows", "amd64"}: "holstered-windows-amd64.exe",
	} {
		if got, err := assetName(in[0], in[1]); err != nil || got != want {
			t.Errorf("assetName(%v) = %q, %v; want %q", in, got, err, want)
		}
	}
	if _, err := assetName("windows", "arm64"); err == nil {
		t.Error("windows/arm64 has no prebuilt binary; want an error")
	}
}

func TestReleaseVersion(t *testing.T) {
	for v, want := range map[string]bool{
		"v1.3.0": true, "(devel)": false, "": false,
		"v0.0.0-20260926120000-abcdef123456":         false,
		"v1.3.1-0.20260926120000-abcdef123456+dirty": false,
	} {
		if got := release.MatchString(v); got != want {
			t.Errorf("release.MatchString(%q) = %v, want %v", v, got, want)
		}
	}
}

func TestChecksumFor(t *testing.T) {
	sums := "aaa  holstered-linux-amd64\nbbb *holstered-windows-amd64.exe\n"
	if got, _ := checksumFor(sums, "holstered-windows-amd64.exe"); got != "bbb" {
		t.Errorf("got %q, want bbb", got)
	}
	if _, err := checksumFor(sums, "holstered-darwin-arm64"); err == nil {
		t.Error("missing asset: want an error")
	}
}

func TestEnsureBinaryDownloadsVerifiesAndCaches(t *testing.T) {
	asset, err := assetName(runtime.GOOS, runtime.GOARCH)
	if err != nil {
		t.Skip(err)
	}
	payload := []byte("#!/bin/sh\necho ok\n")
	sum := sha256.Sum256(payload)
	hits := 0
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		hits++
		switch r.URL.Path {
		case "/download/v9.9.9/SHA256SUMS":
			fmt.Fprintf(w, "%s  %s\n", hex.EncodeToString(sum[:]), asset)
		case "/download/v9.9.9/" + asset:
			w.Write(payload)
		case "/download/v6.6.6/SHA256SUMS":
			fmt.Fprintf(w, "%s  %s\n", strings.Repeat("0", 64), asset)
		case "/download/v6.6.6/" + asset:
			w.Write(payload)
		default:
			http.NotFound(w, r)
		}
	}))
	defer srv.Close()
	releases = srv.URL
	home := t.TempDir()
	t.Setenv("HOME", home)
	t.Setenv("XDG_CACHE_HOME", home)
	t.Setenv("LocalAppData", home)

	bin, err := ensureBinary("v9.9.9")
	if err != nil {
		t.Fatal(err)
	}
	if got, _ := os.ReadFile(bin); string(got) != string(payload) {
		t.Fatalf("cached binary = %q", got)
	}
	if info, _ := os.Stat(bin); runtime.GOOS != "windows" && info.Mode()&0o100 == 0 {
		t.Errorf("cached binary not executable: %v", info.Mode())
	}
	before := hits
	if _, err := ensureBinary("v9.9.9"); err != nil || hits != before {
		t.Errorf("second call should use the cache: err=%v, extra requests=%d", err, hits-before)
	}
	if _, err := ensureBinary("v6.6.6"); err == nil || !strings.Contains(err.Error(), "checksum mismatch") {
		t.Errorf("tampered asset: err = %v, want checksum mismatch", err)
	}
	if _, err := ensureBinary(""); err == nil {
		t.Error("unknown version: want an error")
	}
}
