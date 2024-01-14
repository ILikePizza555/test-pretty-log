{
    inputs = {
        nixpkgs.url = "nixpkgs/nixos-23.11";
        rust-overlay.url = "github:oxalica/rust-overlay";
        flake-utils.url  = "github:numtide/flake-utils";
    };

    outputs = inputs@{self, nixpkgs, rust-overlay, flake-utils}: 
        flake-utils.lib.eachDefaultSystem (system: 
            let
                overlays = [ (import rust-overlay) ];
                pkgs = import nixpkgs { inherit system overlays; };
                rust = (pkgs.rust-bin.stable.latest.default.override {
                            extensions = [ "rust-src" ];
                        });
            in
            {
                devShells.default = pkgs.mkShell {
                    buildInputs = with pkgs; [
                        nixd
                        rust
                    ];
                };
            }
        );
}