variable "environment" {
  description = "Environment name"
  type        = string
  default     = "prod"
}

variable "acm_cert_arn" {
  description = "ACM certificate ARN for SSL"
  type        = string
}

variable "redis_password" {
  description = "Redis authentication password"
  type        = string
  sensitive   = true
}

variable "container_image" {
  description = "ECR image tag for gateway"
  type        = string
  default     = "latest"
}

variable "alert_email" {
  description = "Email for production alerts"
  type        = string
}