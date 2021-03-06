{ sources ? import ./nix/sources.nix }:
let pkgs = import sources.nixpkgs {
  overlays = [
    (import sources.nixpkgs-mozilla)
  ];
};
in with pkgs;
let
  rust-channel = rustChannelOf {
    rustToolChain = ./rust-toolchain;
  };
in mkShell {
  name = "elderscrolls-rust";
  buildInputs = [
    (rust-channel.rust.override {
      extensions = [
        "rust-src"
        "rls-preview"
        "rust-analyzer-preview"
      ];
    })
  ];
}
