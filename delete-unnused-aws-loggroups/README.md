# delete-unnused-aws-loggroups

This tool removes log groups created by AWS services which
are no longer in use. This is done by checking whether the
resources for a given Loggroup still exists (like a Lambda
function). Loggroups created by other services (with other prefixes or no prefixes) are left untouched.

Currently supported servics:
* Lambda (`/aws/lambda/`)
* CodeBuild (`/aws/codebuild/`)

## Setup

This lambda requires the following IAM Policy to be able to list Cloudwatch LogGroups,
Lambdas and CodeBuild Projects, as well as delete CloudWatch LogGroups.

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "logs:CreateLogStream",
                "logs:PutLogEvents"
            ],
            "Resource": [
                "arn:aws:logs:{region}:{account_id}:log-group:${lambda_name}:log-stream:*",
                "arn:aws:logs:{region}:{account_id}:log-group:${lambda_name}"
            ]
        },
        {
            "Effect": "Allow",
            "Action": [
                "logs:DeleteLogGroup",
                "logs:DescribeLogGroups"
            ],
            "Resource": "*"
        },
        {
            "Effect": "Allow",
            "Action": [
                "lambda:ListFunctions"
            ],
            "Resource": "*"
        },
        {
            "Effect": "Allow",
            "Action": [
                "codebuild:ListProjects"
            ],
            "Resource": "*"
        }
    ]
}
```

## Parameters

The lambda function has the following parameters. You can define
them via environment variables.

### Environment Variables
```sh
# Optional, skip if not required. off | error | warn | info (default) | debug | trace
# Defines the log level
LOG_LEVEL=""
```

License: MIT OR Apache-2.0
