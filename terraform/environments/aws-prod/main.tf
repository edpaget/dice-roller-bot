provider "aws" {
  region     = "us-east-2"
}

module "dynamo_table" {
  source = "./stack/ddb/"
}
