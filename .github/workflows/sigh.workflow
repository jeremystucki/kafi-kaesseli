workflow "on push" {
  on = "push"
  resolves = ["python:3"]
}

action "python:3" {
  uses = "python:3"
  runs = "echo \"Hello\""
}
