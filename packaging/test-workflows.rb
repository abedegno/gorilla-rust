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

if failures.empty?
  puts "workflows: ok"
else
  failures.each { |f| warn "FAIL: #{f}" }
  exit 1
end
