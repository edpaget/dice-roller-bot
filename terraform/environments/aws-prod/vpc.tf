locals {
  vpc_id = "vpc-0e14b43fb5e5cf7b2"
}

data "aws_subnets" "vpc_subnets" {
  filter {
    name   = "vpc-id"
    values = [local.vpc_id]
  }
}

resource "aws_security_group" "dice_roller" {
  name        = "prod-dice-roller-sg"
  description = "permit external egress from dice-roller"
  vpc_id      = local.vpc_id
}

resource "aws_vpc_security_group_egress_rule" "allow_all_traffic_ipv4" {
  security_group_id = aws_security_group.dice_roller.id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "-1"
}

resource "aws_vpc_security_group_egress_rule" "allow_all_traffic_ipv6" {
  security_group_id = aws_security_group.dice_roller.id
  cidr_ipv6         = "::/0"
  ip_protocol       = "-1"
}
