resource "aws_cloudwatch_event_bus" "slack_webhook" {
  name = "slack-webhook"
}

resource "aws_cloudwatch_event_archive" "slack_webhook" {
  name             = "slack-webhook"
  event_source_arn = aws_cloudwatch_event_bus.slack_webhook.arn
  retention_days   = 90
}

resource "aws_cloudwatch_event_rule" "slack_webhook" {
  name = "capture-slack-webhook-events"
  event_bus_name = aws_cloudwatch_event_bus.slack_webhook.arn

  event_pattern = jsonencode(
    {
      "source" : ["slack-webhook-to-eventbridge"],
      "detail-type" : [{ "prefix" : "slack_event:" }]
    }
  )
}

resource "aws_cloudwatch_event_target" "slack_webhook" {
  rule           = aws_cloudwatch_event_rule.slack_webhook.name
  event_bus_name = aws_cloudwatch_event_bus.slack_webhook.arn
  target_id      = "SNS"
  arn            = aws_sns_topic.slack_webhook.arn
}
