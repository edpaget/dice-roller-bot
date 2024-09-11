resource "aws_vpc" "prod_dice_roller_vpc" {
  cidr_block = "10.0.0.0/16"
}

resource "aws_subnet" "prod_dice_roller_subnet_a" {
  vpc_id            = aws_vpc.prod_dice_roller_vpc.id
  cidr_block        = "10.0.3.0/24"
  availability_zone = "us-east-2a"
}

resource "aws_subnet" "prod_dice_roller_subnet_b" {
  vpc_id            = aws_vpc.prod_dice_roller_vpc.id
  cidr_block        = "10.0.4.0/24"
  availability_zone = "us-east-2b"
}

resource "aws_egress_only_internet_gateway" "prod_dice_roller_igw" {
  vpc_id = aws_vpc.prod_dice_roller_vpc.id
}

resource "aws_route_table" "prod_dice" {
  vpc_id = aws_vpc.prod_dice_roller_vpc.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_egress_only_internet_gateway.prod_dice_roller_igw.id
  }
}

resource "aws_security_group" "dice_roller" {
  name        = "prod_dice_rollersg"
  description = "permit external egress from dice-roller"
  vpc_id      = aws_vpc.prod_dice_roller_vpc.id
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
