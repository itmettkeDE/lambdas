# export-loggroups-to-s3

This tool exports CloudWatch LogGroups to S3 once a day

## Limitations

This lambda is able to backup between 17_280 and 86_400 log groups. The actual value
depends on how long it takes to backup a single log group. This restriction is based
on the limitation of AWS only allowing a single S3 export at a time per region and
account.

## Setup

### Lambda IAM Policy

This lambda requires the following IAM Policy to be able to list Cloudwatch LogGroups
and to list and create export tasks. It also requires permissions to call itself to
keep on running until all log groups were exported.

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "logs:CreateExportTask",
                "logs:DescribeExportTasks",
                "logs:DescribeLogGroups",
                "logs:DescribeLogStreams",
                "logs:ListTagsLogGroup"
            ],
            "Resource": "*"
        },{
            "Effect": "Allow",
            "Action": [
                "s3:PutObject"
            ],
            "Resource": "arn:aws:s3:::<bucket_name>/<prefix>/*"
        },{
           "Effect": "Allow",
            "Action": [
                "lambda:InvokeAsync"
            ],
            "Resource": "<lambda_arn>"
        }
    ]
}
```

### Bucket Policy

The bucket also requires a policy to allow exports

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "AWSAclCheck",
            "Effect": "Allow",
            "Action": "s3:GetBucketAcl",
            "Resource": "arn:aws:s3:::<bucket_name>",
            "Principal": {
                "Service": [
                    "logs.<region>.amazonaws.com"
                ]
            },
            "Condition": {
                "StringEquals": {
                    "aws:SourceAccount": "<SourceAccountID>"
                }
            }
        },
        {
            "Sid": "AWSCloudWatchWrite",
            "Effect": "Allow",
            "Action": "s3:PutObject",
            "Resource": "arn:aws:s3:::<bucket_name>/<prefix>/*",
            "Principal": {
                "Service": [
                    "logs.<region>.amazonaws.com"
                ]
            },
            "Condition": {
                "StringEquals": {
                    "s3:x-amz-acl": "bucket-owner-full-control",
                    "aws:SourceAccount": "<SourceAccountID>"
                }
            }
        },
        {
            "Sid": "AWSCloudWatchIamWrite",
            "Effect": "Allow",
            "Action": "s3:PutObject",
            "Resource": "arn:aws:s3:::<bucket_name>/<prefix>/*",
            "Principal": {
                "AWS": "<lambda_iam_user>"
            },
            "Condition": {
                "StringEquals": {
                    "s3:x-amz-acl": "bucket-owner-full-control"
                }
            }
        }
    ]
}
```

### Lambda Configuration

For the correct operation of the lambda function two configurations are required:
* Lambda timeout: 15 minutes
* Lambda execution cron: 12pm UTC

The function automatically calls itself after 10 minutes if there are more LogGroups
to backup (it will run at most 24h to not interfere with the next backup round). For
this to work the timeout must be greater then 10 minutes to allow the recursion to
happen.

The execution time is imporant as this lambda will always backup logs from 12:00am UTC to
11:59pm UTC. The [AWS docu](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/S3Export.html)
states that it may take up to 12 hours for logs to be ready for export. That is why the
lambda should run 12 hours after the creation of the last log entry that is to be exported.

## Parameters

The lambda function requires a few parameters to correctly work. You can define
them either with the Event that is send to the lambda, or via environment variables.

The `bucket_prefix` allows these variables:
* `{region}`: Region of the log group
* `{group}`: Name of the log group
* `{year}`: Year of the log entry creation
* `{month}`: Month of the log entry creation
* `{day}`: Day of the log entry creation

### Event

```js
{
    "bucket": "<bucket_name>",
    "prefix": "<bucket_prefix>",
    // Optional, skip if not required. Example: `tag1=value1,tag2=value2`
    // Only include cloudwatch groups with the given tags and values
    "include_tags": [{
        "name": "",
        "value": ""
    }],
    // Optional, skip if not required. Example: `tag1=value1,tag2=value2`
    // Exclude cloudwatch groups with the given tags and values
    "exclude_tags": [{
        "name": "",
        "value": ""
    }]
}
```

### Environment Variables
```sh
BUCKET="<bucket_name>"
PREFIX="<bucket_prefix>"
# Optional, skip if not required. off | error | warn | info (default) | debug | trace
# Defines the log level
LOG_LEVEL=""
# Optional, skip if not required. Example: `tag1=value1,tag2=value2`
# Only include cloudwatch groups with the given tags and values
INCLUDE_TAGS=""
# Optional, skip if not required. Example: `tag1=value1,tag2=value2`
# Exclude cloudwatch groups with the given tags and values
EXCLUDE_TAGS=""
```

License: MIT OR Apache-2.0
