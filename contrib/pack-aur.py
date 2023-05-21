# Reference for later: https://manojkarthick.com/posts/2021/03/rust-aur/
# You should upload the .tar.gz to a release afterwards!

import os
import subprocess
import toml

AUR = "contrib/aur"


class Packer:
    def __init__(self):
        if not os.path.exists("Cargo.toml"):
            print("pack-aur should be called at the root of the Rust project")
            exit(1)

        self.config = toml.load("Cargo.toml")

    def clone_aur(self):
        if not os.path.exists(AUR):
            print(f":: Cloning AUR package")
            subprocess.run(
                ["git", "clone", "ssh://aur@aur.archlinux.org/tori-bin.git", AUR]
            )

    def build_binary(self):
        name = self.config["package"]["name"]
        print(f":: Building \x1b[92m{name}\x1b[0m")  # ]]

        subprocess.run(["cargo", "build", "--release"])
        subprocess.run(["strip", f"target/release/{name}"])

    def make_pkgbuild(self):
        print(":: Generating \x1b[92mPKGBUILD\x1b[0m")  # ]]

        pkg = self.config["package"]
        pkgname = pkg["name"]
        version = pkg["version"]
        description = pkg["description"]
        author0 = pkg["authors"][0]
        license = pkg["license"]
        url = pkg["repository"]
        depends = ['"' + d + '"' for d in pkg["metadata"]["depends"]]
        optdepends = ['"' + d + '"' for d in pkg["metadata"]["optdepends"]]

        content = f"""\
# Maintainer: {author0}

pkgname={pkgname}-bin
pkgver={version}
pkgrel=1
pkgdesc="{description}"
url="{url}"
license=("{license}")
arch=("x86_64")
provides=("{pkgname}")
conflicts=("{pkgname}")
depends=({str.join(" ", depends)})
optdepends=({str.join(" ", optdepends)})
source=("{url}/releases/download/v$pkgver/tori-$pkgver-x86_64.tar.gz")
sha256sums=("we'll see")

package() {{
    install -dm755 "$pkgdir/usr/bin"
    install -dm755 "$pkgdir/usr/share/licenses/$pkgname"
    install -dm755 "$pkgdir/usr/share/applications"
    install -dm755 "$pkgdir/usr/share/pixmaps"

    install -Dm755 tori -t "$pkgdir/usr/bin"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

    install -Dm644 tori.desktop "$pkgdir/usr/share/applications/tori.desktop"
    install -Dm644 tori.svg "$pkgdir/usr/share/pixmaps/"
}}
"""

        with open(f"{AUR}/PKGBUILD", "w") as f:
            f.write(content)

    def make_targz(self):
        pkgname = self.config["package"]["name"]
        version = self.config["package"]["version"]
        filename = f"{pkgname}-{version}-x86_64.tar.gz"
        print(f":: Generating \x1b[92m{filename}\x1b[0m")  # ]]

        subprocess.run(
            [
                "tar",
                "-czf",
                f"{AUR}/{filename}",
                "LICENSE",
                "-C",
                "target/release",
                f"{pkgname}",
                "-C",
                "../../assets",
                "tori.svg",
                "-C",
                "../contrib",
                "tori.desktop",
            ]
        )
        subprocess.run(["updpkgsums", f"{AUR}/PKGBUILD"])

    def makepkg(self):
        print(":: makepkg [.SRCINFO]")
        pwd = os.getcwd()
        os.chdir(AUR)
        with open(".SRCINFO", "w") as srcinfo:
            subprocess.run(["makepkg", "--printsrcinfo"], stdout=srcinfo)

        print(":: makepkg [check]")
        subprocess.run("rm -rf src pkg *.zst", shell=True)  # clean
        subprocess.run(["makepkg"])
        os.chdir(pwd)

    def run(self):
        self.clone_aur()
        self.build_binary()
        self.make_pkgbuild()
        self.make_targz()
        self.makepkg()
        print(":: \x1b[92mDone\x1b[0m")  # ]]


if __name__ == "__main__":
    Packer().run()
