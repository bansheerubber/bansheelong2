{ pkgs ? import <nixpkgs> {} }:
let unstable =
  import <nixos-unstable> { config = { allowUnfree = true; }; };
in pkgs.mkShell rec {
  nativeBuildInputs = [ pkgs.pkg-config ];

  buildInputs = with pkgs.buildPackages; [
    unstable.rustup
    libGL
    libGLU
    faad2
    freeglut
    glew
    glfw
    glm
    xorg.libX11.dev
    xorg.libX11
    xorg.libXi
    xorg.libXi.dev
    libxkbcommon
    libxkbcommon.dev
    xorg.libXcursor
    xorg.libXcursor.dev
    xorg.libXrandr
    xorg.libXft
    xorg.libXinerama
    xorg.libxcb
    xorg.libxcb.dev
    openssl
  ];

  shellHook = ''
    rustup default nightly
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}"
  '';
}
