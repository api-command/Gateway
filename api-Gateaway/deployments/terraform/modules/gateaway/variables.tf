variable "environment" {
  description = "Environment name"
  type        = string
}

variable "subnet_ids" {
  description = "Subnet IDs for ECS tasks"
  type        = list(string)
}

variable "security_group_id" {
  description = "Security Group ID"
  type        = string
}

variable "container_image" {
  description = "ECR image URI"
  type        = string
}

variable "desired_count" {
  description = "Number of ECS tasks"
  type        = number
  default     = 2
}

variable "redis_host" {
  description = "Redis endpoint"
  type        = string
}

variable "target_group_arn" {
  description = "ALB target group ARN"
  type        = string
}