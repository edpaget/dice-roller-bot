data "aws_region" "current" {}
data "aws_partition" "current" {}
data "aws_caller_identity" "current" {}

resource "aws_ecs_cluster" "prod_dice_roller" {
  name = "prod-dice-roller-ecs-cluster"
}

resource "aws_ecs_task_definition" "dice_roller_task" {
  family = "dice-roller-ecs-task-definition"
  track_latest = true

  container_definitions = jsonencode([{
    name = "dice-roller-container"
    essential = true
    image = "${data.aws_ecr_repository.dice_roller.repository_url}:main"
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        awslogs-group = "/prod/dice-roller"
        awslogs-region = "us-east-2"
        awslogs-stream-prefix = "ecs"
      }
    }
    secrets = [
      {
        name = "DISCORD_TOKEN"
        valueFrom = "arn:aws:ssm:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:parameter/prod.discord_bot_token"
      }
    ]
  }])

  cpu = 256
  memory = 512
  network_mode = "awsvpc"

  requires_compatibilities = [
    "FARGATE"
  ]

  execution_role_arn = "arn:aws:iam::311245061868:role/ecsTaskExecutionRole"
  task_role_arn = "arn:aws:iam::311245061868:role/prod-dicer-roller-iam-role"

}

resource "aws_ecs_service" "dice_roller" {
  name            = "dice-roller-ecs-service"
  cluster         = aws_ecs_cluster.prod_dice_roller.id
  task_definition = aws_ecs_task_definition.dice_roller_task.arn
  desired_count   = 1
  launch_type     = "FARGATE"

  network_configuration {
    subnets = data.aws_subnets.vpc_subnets.ids
    security_groups = [aws_security_group.dice_roller.id]
    assign_public_ip = true
  }
}
