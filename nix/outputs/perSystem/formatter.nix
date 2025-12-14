{
  __inputs = {
    imp-fmt.url = "github:imp-nix/imp.fmt";
    imp-fmt.inputs.nixpkgs.follows = "nixpkgs";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    imp-fmt.inputs.treefmt-nix.follows = "treefmt-nix";
  };

  __functor =
    _:
    {
      pkgs,
      treefmt-nix,
      imp-fmt-lib,
      ...
    }:
    imp-fmt-lib.make {
      inherit pkgs treefmt-nix;
      excludes = [
        "target/*"
        "**/target/*"
      ];
      rust.enable = true;
    };
}
