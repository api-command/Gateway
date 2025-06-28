# modules/gateway/locals.tf
locals {
  # Standardized naming convention
  name_prefix = "${var.environment}-gateway"

  # Common tags for all resources
  default_tags = merge(
    var.tags,
    {
      Component   = "api-gateway"
      Environment = var.environment
      ManagedBy   = "terraform"
      Domain      = "edge-services"
    }
  )

  # CloudWatch log group configuration
  log_group_name = "/ecs/${local.name_prefix}"

  # IAM role names
  iam_role_names = {
    execution = "${local.name_prefix}-execution-role"
    task      = "${local.name_prefix}-task-role"
  }

  # Auto Scaling configuration
  autoscaling = {
    metric_name = "ECSServiceAverageCPUUtilization"
    target_value = 70
    scale_in_cooldown  = 300
    scale_out_cooldown = 60
  }

  # Container health check parameters
  health_check = {
    command     = ["CMD-SHELL", "curl -f http://localhost:${var.container_port}/health || exit 1"]
    interval    = 30
    timeout     = 5
    retries     = 3
    startPeriod = 60
  }

  # Network configuration
  network_settings = {
    assign_public_ip = var.environment == "prod" ? false : true
    protocol         = "tcp"
  }

  # Secret ARN map (SSM Parameter Store paths)
  secret_arns = {
    redis_host = var.redis_host_arn
    jwt_secret = var.jwt_secret_arn
    # Add other secrets as needed
  }
}