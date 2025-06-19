{ stdenv
, fetchurl
, lib
}:

stdenv.mkDerivation rec {
  pname = "wasi-sdk";
  version = "25.0-x86_64-linux";

  src = fetchurl {
    url = "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-25/wasi-sdk-${version}.tar.gz";
    sha256 = "sha256-UmQN3hNZm/EnqVSZ5h1tZAJWEZRW0a+Il6tnJbzz2Jw=";
  };

  dontBuild = true;

  phases = [ "unpackPhase" ];
  unpackPhase = ''
    mkdir -p $out
    tar xzf $src -C $out
  '';

  meta = with lib; {
    description = "WASI SDK 25.0 for aarch64-linux";
    homepage = "https://github.com/WebAssembly/wasi-sdk";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
