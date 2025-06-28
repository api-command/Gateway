variable "acm_cert_arn" {
  description = "ACM certificate ARN for SSL"
  type        = string
}

variable "redis_password" {
  description = "Redis authentication password"
  type        = string
  sensitive   = true
}