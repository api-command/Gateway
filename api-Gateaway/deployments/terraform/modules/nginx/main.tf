# modules/nginx/main.tf
resource "aws_lb" "gateway" {
  name               = "${var.environment}-api-gw-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = var.public_subnet_ids

  enable_deletion_protection = var.environment == "prod" ? true : false
  enable_http2               = true
  drop_invalid_header_fields = true

  access_logs {
    bucket  = var.log_bucket_name
    prefix  = "alb/${var.environment}"
    enabled = true
  }

  tags = merge(var.tags, {
    Component = "load-balancer"
  })
}

resource "aws_lb_target_group" "gateway" {
  name        = "${var.environment}-api-gw-tg"
  port        = var.backend_port
  protocol    = "HTTP"
  vpc_id      = var.vpc_id
  target_type = "ip"

  health_check {
    path                = "/health"
    protocol            = "HTTP"
    interval            = 30
    timeout             = 5
    healthy_threshold   = 2
    unhealthy_threshold = 2
    matcher             = "200-299"
  }

  stickiness {
    type    = "lb_cookie"
    enabled = var.environment == "prod" ? true : false
  }

  tags = var.tags
}

resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.gateway.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type = "redirect"

    redirect {
      port        = "443"
      protocol    = "HTTPS"
      status_code = "HTTP_301"
    }
  }
}

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.gateway.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.acm_cert_arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.gateway.arn
  }
}

resource "aws_security_group" "alb" {
  name        = "${var.environment}-alb-sg"
  description = "Security group for API Gateway ALB"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = var.backend_port
    to_port     = var.backend_port
    protocol    = "tcp"
    cidr_blocks = var.private_subnet_cidrs
  }

  tags = merge(var.tags, {
    Component = "security-group"
  })
}

resource "aws_wafv2_web_acl_association" "alb" {
  count        = var.enable_waf ? 1 : 0
  resource_arn = aws_lb.gateway.arn
  web_acl_arn  = var.waf_acl_arn
}

resource "aws_cloudwatch_metric_alarm" "alb_5xx" {
  alarm_name          = "${var.environment}-alb-5xx-errors"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "HTTPCode_ELB_5XX_Count"
  namespace           = "AWS/ApplicationELB"
  period              = "300"
  statistic           = "Sum"
  threshold           = "10"
  alarm_description   = "ALB 5XX error rate exceeded threshold"
  alarm_actions       = var.alarm_actions

  dimensions = {
    LoadBalancer = aws_lb.gateway.arn_suffix
  }

  tags = var.tags
}