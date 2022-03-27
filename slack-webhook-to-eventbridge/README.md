# slack-webhook-to-eventbridge

This Lambda is compatible with AWS WebAPI. Its supposed to be connected to the Slack Event Subscription and will send incoming Slack Events to EventBridge.

## Setup

* Create a lambda with the binary from this repository using runtime `provided.al2`
and anything as handler. (More Infos about paramters below).
* Add permissions for `secretsmanager:GetSecretValue` and `events:PutEvents` to the
lambdas iam role

## Parameters

The lambda function has a few parameters. You can define them via
environment variables.

### Environment Variables
```sh
# AWS Secretsmanager Secret which contains the signing_secret in the following format:
# `{"signing_secret": ""}`
SLACK_SECRET=""
# Name of the AWS Eventbridge this lambda is supposed to send events to
EVENTBRIDGE_BUS_NAME=""
# Optional, skip if not required. off | error | warn | info (default) | debug | trace
# Defines the log level
LOG_LEVEL=""
```

License: MIT OR Apache-2.0
