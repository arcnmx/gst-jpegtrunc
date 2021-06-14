{ stdenv
, nix-gitignore
, gst_all_1
, gstreamer ? gst_all_1.gstreamer
, pkg-config
, rustPlatform
, buildType ? "release"
}: rustPlatform.buildRustPackage {
  pname = "gst-jpegtrunc";
  version = "0.1.0";
  inherit buildType;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ gstreamer ];

  src = nix-gitignore.gitignoreSource [ ''
    /*.nix
    /.github/
    /testdata/
  '' ] ./.;

  cargoSha256 = "0aivcv83f7nixwj8kdfghi482r8kai2fsas8mkr5wyyg6iby22pj";

  libname = "libgstjpegtrunc" + stdenv.hostPlatform.extensions.sharedLibrary;

  postInstall = ''
    mkdir -p $out/lib/gstreamer-1.0
    mv $out/lib/$libname $out/lib/gstreamer-1.0/
  '';
}
