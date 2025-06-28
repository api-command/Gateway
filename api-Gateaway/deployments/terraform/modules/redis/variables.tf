variable "environment" {
  description = "Environment name"
  type        = string
}

variable "subnet_ids" {
  description = "Subnet IDs for Redis cluster"
  type        = list(string)
}

variable "security_group_id" {
  description = "Security Group ID"
  type        = string
}

variable "node_type" {
  description = "Redis node type"
  type        = string
  default     = "cache.t3.micro"
}

variable "redis_password" {
  description = "Redis auth password"
  type        = string
  sensitive   = true
}