{
  description = "Dev shell for Bevy projects with Rust tools";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; };
    in
    {
      devShell.x86_64-linux = pkgs.mkShell {
        nativeBuildInputs = [
          pkgs.pkg-config
        ];

        # Define dependencies in a single list
        buildInputs = with pkgs; [
          udev
          alsa-lib
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          libxkbcommon
          wayland
          rustc
          cargo
        ];

        # Set library paths
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
          udev
          alsa-lib
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          libxkbcommon
          wayland
        ]);

        # Optional shell hook for Rust tools
        shellHook = ''
          echo "Rust tools (cargo, rustc) and Bevy dependencies are ready to use!"
        '';
      };
    };
}
