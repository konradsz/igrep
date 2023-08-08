class Igrep < Formula
  version "1.2.0"
  desc "Interactive Grep"
  homepage "https://github.com/konradsz/igrep"
  url "https://github.com/konradsz/igrep/releases/download/v#{version}/igrep-v#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "f987d7659654b45d84989dd6cf8a491aaba70369b0caf4d5cb8fd88294885ed0"

  def install
    bin.install "ig"
  end
end
