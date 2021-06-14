{ pkgs ? import <nixpkgs> { } }: with pkgs; with gst_all_1; let
  gst-jpegtrunc = pkgs.callPackage ./derivation.nix { };
  gst-jpegtrunc-debug = gst-jpegtrunc.override { buildType = "debug"; };
  testCommand = writeShellScriptBin "runtest" ''
    ${./testdata/runtest.sh} ${./testdata/input.jpg}
  '';
  testshell = mkShell {
    GST_PLUGIN_SYSTEM_PATH_1_0 = lib.makeSearchPath "lib/gstreamer-1.0" [
      gstreamer
      gst-plugins-base
      gst-jpegtrunc-debug
    ];
    GST_DEBUG = "DEBUG"; # WARNING/LOG/TRACE?
    nativeBuildInputs = [
      testCommand
      gstreamer.dev
    ];
  };
  rustShell = shells.rust.nightly.overrideAttrs (old: {
    nativeBuildInputs = old.nativeBuildInputs or [ ] ++ gst-jpegtrunc.nativeBuildInputs;
    buildInputs = old.buildInputs or [ ] ++ gst-jpegtrunc.buildInputs;
  });
in gst-jpegtrunc // {
  inherit testshell;
  shell = if pkgs ? shells.rust then rustShell else gst-jpegtrunc;
  debug = gst-jpegtrunc-debug;
}
