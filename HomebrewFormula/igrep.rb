class Igrep < Formula
  version "0.3.0"
  desc "Interactive Grep"
  homepage "https://github.com/konradsz/igrep"
  url "https://github.com/konradsz/igrep/releases/download/v#{version}/igrep-v#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "127f05a7e14c1031b202f5b3e39a5bbfde28b93cf4d700dde8310f52276189f8"

  def install
    bin.install "ig"
  end
end
