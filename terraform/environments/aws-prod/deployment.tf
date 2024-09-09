locals {
  provider_url = "token.actions.githubusercontent.com"
  audience     = "sts.amazonaws.com"
  subject      = "repo:edpaget/dice-roller-bot:*"

  account_id = data.aws_caller_identity.current.account_id
}

data "aws_iam_policy_document" "dice_roller_github_oidc_assume_policy" {
  statement {
    sid    = "GithubOidcAuth"
    effect = "Allow"
    actions = [
      "sts:TagSession",
      "sts:AssumeRoleWithWebIdentity"
    ]

    principals {
      type        = "Federated"
      identifiers = ["arn:aws:iam::${local.account_id}:oidc-provider/${local.provider_url}"]
    }

    condition {
      test     = "ForAllValues:StringEquals"
      variable = "${local.provider_url}:iss"
      values   = ["https://${local.provider_url}"]
    }

    condition {
      test     = "ForAllValues:StringEquals"
      variable = "${local.provider_url}:aud"
      values   = [local.audience]
    }

    condition {
      test     = "StringLike"
      variable = "${local.provider_url}:sub"
      values   = [local.subject]
    }
  }
}

resource "aws_iam_role" "dice_roller_github_oidc_deployment_role" {
  name        = "dice-roller-github-oidc-deployment-role"
  description = "role for github oidc ecr/ecs deployment actions for dice-roller"

  assume_role_policy = data.aws_iam_policy_document.dice_roller_github_oidc_assume_policy.json
}

resource "aws_iam_role_policy_attachment" "dice_roller_ecr_deployment_policy_attachment" {
  role       = aws_iam_role.dice_roller_github_oidc_deployment_role.name
  policy_arn = aws_iam_policy.dice_roller_ecr_deployment_policy.arn
}

resource "aws_iam_policy" "dice_roller_ecr_deployment_policy" {
  name        = "dice-roller-ecr-deployment-policy"
  description = "allow ecr deployment by github oidc"

  policy = data.aws_iam_policy_document.dice_roller_ecr_deployment_policy_doc.json

}

data "aws_iam_policy_document" "dice_roller_ecr_deployment_policy_doc" {
  statement {
    sid    = "ECRImagePush"
    effect = "Allow"
    actions = [
      "ecr:CompleteLayerUpload",
      "ecr:UploadLayerPart",
      "ecr:InitiateLayerUpload",
      "ecr:BatchCheckLayerAvailability",
      "ecr:PutImage",
    ]
    resources = [data.aws_ecr_repository.dice_roller.arn]
  }

  statement {
    sid    = "AllowECRLogin"
    effect = "Allow"
    actions = [
      "ecr:GetAuthorizationToken"
    ]
    resources = ["*"]
  }
}

resource "aws_iam_role_policy_attachment" "dice_roller_ecs_deployment_policy_attachment" {
  role       = aws_iam_role.dice_roller_github_oidc_deployment_role.name
  policy_arn = aws_iam_policy.dice_roller_ecs_deployment_policy.arn
}

resource "aws_iam_policy" "dice_roller_ecs_deployment_policy" {
  name        = "dice-roller-ecs-deployment-policy"
  description = "allow ecs deployment by github oidc"

  policy = data.aws_iam_policy_document.dice_roller_ecs_deployment_policy_doc.json

}

data "aws_iam_policy_document" "dice_roller_ecs_deployment_policy_doc" {
  statement {
    sid    = "RegisterTaskDefinition"
    effect = "Allow"
    actions = [
      "ecs:RegisterTaskDefinition"
    ]
    resources = ["*"]
  }

  statement {
    sid    = "GetTaskDefinition"
    effect = "Allow"
    actions = [
      "ecs:DescribeTaskDefinition"
    ]
    resources = [
      "${aws_ecs_task_definition.dice_roller_task.arn_without_revision}:*",
    ]
  }

  statement {
    sid    = "PassRolesInTaskDefinition"
    effect = "Allow"
    actions = [
      "iam:PassRole"
    ]
    resources = [
      "arn:aws:iam::311245061868:role/ecsTaskExecutionRole",
      "arn:aws:iam::311245061868:role/prod-dicer-roller-iam-role",
    ]
  }

  statement {
    sid    = "DeployService"
    effect = "Allow"
    actions = [
      "ecs:UpdateService",
      "ecs:DescribeServices"
    ]
    resources = [
      aws_ecs_service.dice_roller.id
    ]
  }
}
