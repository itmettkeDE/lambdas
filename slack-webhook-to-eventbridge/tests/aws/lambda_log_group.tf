resource "aws_cloudwatch_log_group" "slack_webhook_arm64" {
  name              = "/aws/lambda/${aws_lambda_function.slack_webhook_arm64.function_name}"
  retention_in_days = 7
}

resource "aws_cloudwatch_log_group" "slack_webhook_x86_64" {
  name              = "/aws/lambda/${aws_lambda_function.slack_webhook_x86_64.function_name}"
  retention_in_days = 7
}
