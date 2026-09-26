# The checks behind test-workflows.sh. Each one pins a mistake that only
# shows on a real release, when it is too late.
require "yaml"

wf = YAML.load_file(ARGV.fetch(0))
jobs = wf.fetch("jobs")
failures = []

# A failing $(...) inside echo's arguments does not stop a step, since the
# step's status is echo's. A tag that disagrees with Cargo.toml would then
# be signed and published with an empty version, so version.sh may only be
# called in an assignment, which does fail.
jobs.each do |name, job|
  (job["steps"] || []).each do |step|
    step["run"].to_s.each_line do |line|
      next unless line.include?("packaging/version.sh")
      next if line.strip.match?(/\A\w+="\$\(packaging\/version\.sh\)"\z/)
      failures << "#{name}: version.sh is not assigned to a variable in: #{line.strip}"
    end
  end
end

# The signing secrets go only to the steps that use them. cargo runs
# third-party build scripts, which must never see the certificate or the
# notary key.
secret = ->(env) { (env || {}).select { |_, v| v.to_s.include?("secrets.") }.keys }
leaked = secret.call(jobs.fetch("macos")["env"])
failures << "the macos job gives every step #{leaked.join(', ')}" unless leaked.empty?
jobs.each do |name, job|
  (job["steps"] || []).each do |step|
    next unless step["run"].to_s.include?("cargo ")
    names = secret.call(step["env"])
    failures << "#{name}: '#{step['name']}' runs cargo with #{names.join(', ')}" unless names.empty?
  end
end

# Re-running the homebrew job, which docs/releasing.md gives as the fix for
# an expired token, uploads its artifact a second time in the same run.
upload = jobs.fetch("homebrew").fetch("steps").find { |s| s["uses"].to_s.start_with?("actions/upload-artifact") }
unless upload && upload.dig("with", "overwrite") == true
  failures << "the homebrew job's artifact upload needs overwrite: true to survive a re-run"
end

all_steps = jobs.flat_map { |name, job| (job["steps"] || []).map { |step| [name, step] } }

# With assessments turned off, spctl says "accepted" with "override=security
# disabled" and exits 0, which proves nothing. Only the source line shows
# that Gatekeeper saw a notarization.
# The default run shell has no pipefail, so without it the grep alone would
# decide, and a rejection that still names a source would pass. grep reads
# to the end rather than quitting with -q, so tee is never cut off.
all_steps.each do |name, step|
  run = step["run"].to_s
  next unless run.include?("spctl --assess")
  failures << "#{name}: '#{step['name']}' runs spctl without set -o pipefail" unless run.include?("set -o pipefail")
  run.each_line do |line|
    next unless line.include?("spctl --assess")
    next if line.include?("| grep 'source=Notarized Developer ID' >/dev/null")
    failures << "#{name}: spctl's verdict is not checked for the notarization in: #{line.strip}"
  end
end

# Under pipefail, a grep that finds no identity fails the assignment and the
# step dies before it can say why.
import = all_steps.map(&:last).find { |s| s["run"].to_s.include?("security find-identity") }
unless import && import["run"].match?(/find-identity.*?\|\| true\)"/m)
  failures << "the identity lookup must tolerate no match so its error message can print"
end

# A re-run of an old release's homebrew job must not put the tap back.
push = jobs.fetch("homebrew").fetch("steps").find { |s| s["name"] == "Push to the tap" }
unless push && push["run"].to_s.include?("packaging/homebrew/tap-is-newer.sh")
  failures << "Push to the tap must skip a tap that already holds a newer release"
end

# Two tags pushed close together must take turns at the tap.
concurrency = jobs.fetch("homebrew")["concurrency"]
unless concurrency.is_a?(Hash) && concurrency["group"] == "homebrew-tap" && concurrency["cancel-in-progress"] == false
  failures << "the homebrew job needs concurrency group homebrew-tap without cancel-in-progress"
end

# A pre-release tag stays out of the tap, so it must not become GitHub's
# latest release either, which the README and the cask's livecheck follow.
publish = jobs.fetch("release").fetch("steps").find { |s| s["uses"].to_s.start_with?("softprops/action-gh-release") }
unless publish && publish.dig("with", "prerelease") == "${{ contains(github.ref_name, '-') }}"
  failures << "the release must be marked a pre-release when its tag has a hyphen"
end

# A moving branch ref runs whatever is pushed there next, with this
# workflow's permissions.
all_steps.each do |name, step|
  uses = step["uses"].to_s
  failures << "#{name}: #{uses} follows a branch" if uses.match?(/@(main|master)\z/)
end

# notarize.sh keeps Apple's verdict in a temporary file, which must go
# whether the submission is accepted or not.
notarize = File.read(File.join(__dir__, "macos", "notarize.sh"))
failures << "notarize.sh must remove its temporary file on every exit" unless notarize.match?(/^trap 'rm -f "\$result"' EXIT$/)

if failures.empty?
  puts "workflows: ok"
else
  failures.each { |f| warn "FAIL: #{f}" }
  exit 1
end
