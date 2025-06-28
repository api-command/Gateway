variable "environment" {
  description = "Environment name"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID"
  type        = string
}

variable "public_subnet_ids" {
  description = "Public subnet IDs"
  type        = list(string)
}

variable "security_group_id" {
  description = "Security Group ID"
  type        = string
}

variable "acm_cert_arn" {
  description = "ACM certificate ARN"
  type        = string
}