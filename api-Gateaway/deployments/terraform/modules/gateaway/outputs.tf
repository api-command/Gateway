output "ecs_service_name" {
  value = aws_ecs_service.gateway.name
}

output "task_definition_arn" {
  value = aws_ecs_task_definition.gateway.arn
}