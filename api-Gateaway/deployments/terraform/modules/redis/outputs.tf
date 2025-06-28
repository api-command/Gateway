output "redis_endpoint" {
  value = aws_elasticache_cluster.gateway.cache_nodes[0].address
}