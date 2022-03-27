data "aws_lambda_invocation" "slack_webhook_arm64" {
  depends_on = [
    aws_cloudwatch_log_group.slack_webhook_arm64,
    aws_secretsmanager_secret_version.slack_secret,
  ]

  function_name = aws_lambda_function.slack_webhook_arm64.function_name

  input = local.event
}

data "aws_lambda_invocation" "slack_webhook_x86_64" {
  depends_on = [
    aws_cloudwatch_log_group.slack_webhook_x86_64,
    aws_secretsmanager_secret_version.slack_secret,
  ]

  function_name = aws_lambda_function.slack_webhook_x86_64.function_name

  input = local.event
}

data "external" "slack_webhook_headers" {
  program = ["bash", "${path.module}/build_headers.sh", local.body, random_password.test_pw.result]
}

locals {
  body = jsonencode(
    {
      "event" : {
        "type" : "TestEvent"
      }
    }
  )
  event = jsonencode(
    {
      "headers" : {
        "X-Slack-Request-Timestamp" : data.external.slack_webhook_headers.result.ts,
        "X-Slack-Signature" : data.external.slack_webhook_headers.result.sig,
      },
      "body" : local.body,
    }
  )
}

output "invoce_ouput" {
  value = {
    "arm64" : data.aws_lambda_invocation.slack_webhook_arm64.result,
    "x86_64" : data.aws_lambda_invocation.slack_webhook_x86_64.result,
  }
}
