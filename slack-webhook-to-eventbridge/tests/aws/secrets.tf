resource "aws_secretsmanager_secret" "slack_secret" {
  name = "slack-secret"
}

resource "random_password" "test_pw" {
  length  = 16
  special = false
}

resource "aws_secretsmanager_secret_version" "slack_secret" {
  secret_id = aws_secretsmanager_secret.slack_secret.id
  secret_string = jsonencode({
    "signing_secret" : random_password.test_pw.result
  })
}
