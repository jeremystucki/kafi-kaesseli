workflow "New workflow" {
  on = "push"
  
  resolves = [
    "args",
  ]
}

action "args" {
  uses = "docker://alpine"
  args = "echo hi"
}
