provider "aws" {
  region     = "us-east-1"
}

module "dynamo_table" {
  source = "./stack/ddb/"
}
