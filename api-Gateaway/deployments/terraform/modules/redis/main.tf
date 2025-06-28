resource "aws_elasticache_cluster" "gateway" {
  cluster_id           = "${var.environment}-redis"
  engine               = "redis"
  node_type            = var.node_type
  num_cache_nodes      = var.environment == "prod" ? 3 : 1
  parameter_group_name = "default.redis6.x"
  engine_version       = "6.x"
  port                 = 6379
  security_group_ids   = [var.security_group_id]
  subnet_group_name    = aws_elasticache_subnet_group.main.name
}

resource "aws_elasticache_subnet_group" "main" {
  name       = "${var.environment}-redis-subnet"
  subnet_ids = var.subnet_ids
}