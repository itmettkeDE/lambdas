# rotate-google-service-user

This tool rotates Google Service User Credentials stored in AWS Secret Manager

Setup

* Create a Google IAM Role which contains the following permissions:
  * `iam.serviceAccountKeys.create`
  * `iam.serviceAccountKeys.delete`
* Attach the role with the following condition to the service user whoms
credentials you want to rotate:
  * `resource.name == "projects/-/serviceAccounts/<service-user-unique-id>"`
* The rotation function requires that the complete credential.json from the
service user is available somewhere in the secret, either as string containing
the json or as json object.
* Create a lambda with the binary from this repository using runtime `provided.al2`
and anything as handler. (More Infos about paramters below)
* Attach the lambda as rotation lambda to the AWS Secret Manager Secret

## Parameters

The lambda function has a few optional parameters. You can define them via
environment variables.

### Environment Variables
```sh
# Optional, skip if not required.
# Defines the keys to traverse to find the credential.json. Example:
# { "test": [<credential.json>] }
# requires : JSON_PATH="[\"test\", 0]"
# default: JSON_PATH="[]" which expectes the credential.json to be at the top of the secret
JSON_PATH="[]"
# Optional, skip if not required. off | error | warn | info (default) | debug | trace
# Defines the log level
LOG_LEVEL=""
```


License: MIT OR Apache-2.0
