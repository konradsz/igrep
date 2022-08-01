class Igrep < Formula
  version "0.5.1"
  desc "Interactive Grep"
  homepage "https://github.com/konradsz/igrep"
  url "https://github.com/konradsz/igrep/releases/download/v#{version}/igrep-v#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "359c90fe0a53dc7416cd838b32279ad48811926e78000a7aa541c8a396dd87ea"

  def install
    bin.install "ig"
  end
end
