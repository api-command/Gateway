variable "environment" {
  description = "Environment name"
  type        = string
  default     = "staging"
}

variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
}

variable "azs" {
  description = "Availability Zones"
  type        = list(string)
}

variable "public_subnet_cidrs" {
  description = "Public subnet CIDR blocks"
  type        = list(string)
}

variable "private_subnet_cidrs" {
  description = "Private subnet CIDR blocks"
  type        = list(string)
}

variable "acm_cert_arn" {
  description = "ACM certificate ARN"
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
}

variable "alert_email" {
  description = "Email for staging alerts"
  type        = string
}