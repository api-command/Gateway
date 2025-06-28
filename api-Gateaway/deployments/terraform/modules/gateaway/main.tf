resource "aws_ecs_cluster" "gateway" {
  name = "${var.environment}-gateway-cluster"
}

resource "aws_ecs_task_definition" "gateway" {
  family                   = "gateway"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = var.task_cpu
  memory                   = var.task_memory
  execution_role_arn       = aws_iam_role.ecs_task_execution.arn

  container_definitions = jsonencode([{
    name  = "gateway"
    image = var.container_image
    portMappings = [{
      containerPort = 8080
      hostPort      = 8080
    }]
    environment = [
      { name = "REDIS_HOST", value = var.redis_host },
      { name = "ENVIRONMENT", value = var.environment }
    ]
  }])
}

resource "aws_ecs_service" "gateway" {
  name            = "gateway-service"
  cluster         = aws_ecs_cluster.gateway.id
  task_definition = aws_ecs_task_definition.gateway.arn
  desired_count   = var.desired_count
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = var.subnet_ids
    security_groups  = [var.security_group_id]
    assign_public_ip = true
  }

  load_balancer {
    target_group_arn = var.target_group_arn
    container_name   = "gateway"
    container_port   = 8080
  }
}