class Igrep < Formula
  version "1.3.0"
  desc "Interactive Grep"
  homepage "https://github.com/konradsz/igrep"
  url "https://github.com/konradsz/igrep/releases/download/v#{version}/igrep-v#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "33908e25d904d7652f2bc749a16beaae86b9529f83aeae8ca6834dddfc2b0a9d"

  def install
    bin.install "ig"
  end
end
