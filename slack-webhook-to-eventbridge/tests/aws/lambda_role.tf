resource "aws_iam_role" "slack_webhook" {
  name = "slack-webhook"

  assume_role_policy = jsonencode(
    {
      "Version" : "2012-10-17",
      "Statement" : [
        {
          "Effect" : "Allow",
          "Action" : "sts:AssumeRole",
          "Principal" : {
            "Service" : "lambda.amazonaws.com"
          },
        }
      ]
    }
  )
}

data "aws_iam_policy" "AWSLambdaBasicExecutionRole" {
  arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy_attachment" "slack_webhook" {
  role       = aws_iam_role.slack_webhook.name
  policy_arn = data.aws_iam_policy.AWSLambdaBasicExecutionRole.arn
}

resource "aws_iam_role_policy" "slack_webhook" {
  name = "slack-webhook-role"
  role = aws_iam_role.slack_webhook.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        "Effect" : "Allow",
        "Action" : "secretsmanager:GetSecretValue",
        "Resource" : aws_secretsmanager_secret.slack_secret.arn,
      },
      {
        "Effect" : "Allow",
        "Action" : "events:PutEvents",
        "Resource" : aws_cloudwatch_event_bus.slack_webhook.arn,
        "Condition" : {
          "StringEquals" : {
            "events:source" : "slack-webhook-to-eventbridge"
          },
          "StringLike" : {
            "events:detail-type" : "slack_event:*"
          },
        }
      }
    ]
  })
}
