class Igrep < Formula
  version "0.5.0"
  desc "Interactive Grep"
  homepage "https://github.com/konradsz/igrep"
  url "https://github.com/konradsz/igrep/releases/download/v#{version}/igrep-v#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "968f83152d9b00ede61e6336baa97193bf778ef36fb8d7a3fe8870e81c6d646b"

  def install
    bin.install "ig"
  end
end
