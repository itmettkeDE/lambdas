resource "aws_lambda_function" "slack_webhook_arm64" {
  function_name = "slack-webhook-to-eventbridge-arm64"
  role          = aws_iam_role.slack_webhook.arn

  architectures = ["arm64"]
  handler       = "unrelevant"
  runtime       = "provided.al2"

  filename         = "slack-webhook-to-eventbridge-aarch64.zip"
  source_code_hash = filebase64sha256("slack-webhook-to-eventbridge-aarch64.zip")

  environment {
    variables = {
      LOG_LEVEL            = "info"
      SLACK_SECRET         = aws_secretsmanager_secret.slack_secret.id
      EVENTBRIDGE_BUS_NAME = aws_cloudwatch_event_bus.slack_webhook.name
    }
  }
}

resource "aws_lambda_function" "slack_webhook_x86_64" {
  function_name = "slack-webhook-to-eventbridge-x86_64"
  role          = aws_iam_role.slack_webhook.arn

  architectures = ["x86_64"]
  handler       = "unrelevant"
  runtime       = "provided.al2"

  filename         = "slack-webhook-to-eventbridge-x86_64.zip"
  source_code_hash = filebase64sha256("slack-webhook-to-eventbridge-x86_64.zip")

  environment {
    variables = {
      LOG_LEVEL            = "info"
      SLACK_SECRET         = aws_secretsmanager_secret.slack_secret.id
      EVENTBRIDGE_BUS_NAME = aws_cloudwatch_event_bus.slack_webhook.name
    }
  }
}
