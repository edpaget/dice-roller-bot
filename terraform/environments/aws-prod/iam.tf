resource "aws_iam_role" "dice_roller_ecs_execution_role" {
  name        = "dice-roller-ecs-execution-role"
  description = "ecs execution role for the dice roller application"

  assume_role_policy = data.aws_iam_policy_document.dice_roller_ecs_task_assume_role_doc.json
}

resource "aws_iam_role" "dice_roller_ecs_task_role" {
  name        = "dice-roller-ecs-task-role"
  description = "ecs task role for the dice roller application"

  assume_role_policy = data.aws_iam_policy_document.dice_roller_ecs_task_assume_role_doc.json
}

data "aws_iam_policy_document" "dice_roller_ecs_task_assume_role_doc" {
  statement {
    sid    = "ecs_task_assume_role"
    effect = "Allow"
    actions = [
      "sts:AssumeRole",
    ]

    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "dice_roller_ecs_execution_policy_doc" {
  statement {
    sid    = "ECSExecutionRole"
    effect = "Allow"
    actions = [
      "ecr:GetAuthorizationToken",
      "ecr:BatchCheckLayerAvailability",
      "ecr:GetDownloadUrlForLayer",
      "ecr:BatchGetImage",
      "logs:CreateLogStream",
      "logs:PutLogEvents"
    ]
    resources = ["*"]
  }

  statement {
    sid    = "GetSSMSecrets"
    effect = "Allow"
    actions = [
      "ssm:GetParameters",
    ]
    resources = [
      "arn:aws:ssm:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:parameter/prod.discord_bot_token"
    ]
  }
}

data "aws_iam_policy_document" "dice_roller_ecs_task_policy_doc" {
  statement {
    sid    = "DynamoDBAccess"
    effect = "Allow"
    actions = [
      "dynamodb:GetItem",
      "dynamodb:PutItem",
      "dynamodb:Query",
    ]
    resources = [
      module.dynamo_table.table_arn,
    ]
  }
}

resource "aws_iam_policy" "dice_roller_ecs_execution_policy" {
  name        = "dice-roller-ecs-execution-policy"
  description = "Allow ecs execution by github oidc"

  policy = data.aws_iam_policy_document.dice_roller_ecs_execution_policy_doc.json

}

resource "aws_iam_policy" "dice_roller_ecs_task_policy" {
  name        = "dice-roller-ecs-task-policy"
  description = "Allow ecs task by github oidc"

  policy = data.aws_iam_policy_document.dice_roller_ecs_task_policy_doc.json

}

resource "aws_iam_role_policy_attachment" "dice_roller_ecs_execution_policy_attachment" {
  role       = aws_iam_role.dice_roller_ecs_execution_role.name
  policy_arn = aws_iam_policy.dice_roller_ecs_execution_policy.arn
}

resource "aws_iam_role_policy_attachment" "dice_roller_ecs_task_policy_attachment" {
  role       = aws_iam_role.dice_roller_ecs_task_role.name
  policy_arn = aws_iam_policy.dice_roller_ecs_task_policy.arn
}
