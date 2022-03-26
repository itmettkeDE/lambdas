resource "aws_sns_topic" "slack_webhook" {
  name = "slack_webhook"
}

resource "aws_sns_topic_subscription" "slack_webhook" {
  topic_arn = aws_sns_topic.slack_webhook.arn
  protocol  = "email"
  endpoint  = "marc@itmettke.de"
}

resource "aws_sns_topic_policy" "slack_webhook" {
  arn = aws_sns_topic.slack_webhook.arn

  policy = data.aws_iam_policy_document.slack_webhook.json
}

data "aws_iam_policy_document" "slack_webhook" {
  policy_id = "__default_policy_ID"

  statement {
    actions = [
      "SNS:Subscribe",
      "SNS:SetTopicAttributes",
      "SNS:RemovePermission",
      "SNS:Receive",
      "SNS:Publish",
      "SNS:ListSubscriptionsByTopic",
      "SNS:GetTopicAttributes",
      "SNS:DeleteTopic",
      "SNS:AddPermission",
    ]

    condition {
      test     = "StringEquals"
      variable = "AWS:SourceOwner"

      values = [
        data.aws_caller_identity.current.account_id,
      ]
    }

    effect = "Allow"

    principals {
      type        = "AWS"
      identifiers = ["*"]
    }

    resources = [
      aws_sns_topic.slack_webhook.arn,
    ]

    sid = "__default_statement_ID"
  }

  statement {
    effect = "Allow"

    principals {
      type        = "Service"
      identifiers = ["events.amazonaws.com"]
    }

    actions = [
      "SNS:Publish",
    ]

    resources = [
      aws_sns_topic.slack_webhook.arn,
    ]

    sid = "__sns_statement_ID"
  }
}
