default: build

all: test

test: build
	cargo test

build:
	stellar contract build
	@ls -l target/wasm32v1-none/release/*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean


create_escrow:
	stellar contract invoke \
  --id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A \
  --source-account maycon \
  -- create_escrow \
  --allow_early_release true \
  --buyer maycon \
  --seller GBBIVZN5N7EMYMQHZL4ME64GWDM5REJDLFBDET7KLIIA6GQRQVJ2IQWE \
  --amount 454354147 \
  --asset CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC \
  --fee_bps 100 \
  --guarantee_days $(days) \
  --product_id cc5c64c7-4392-40da-8959-a8cb61fb5552

get_escrow:
	stellar contract invoke \
  --id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A \
  --source-account maycon \
  -- get_escrow \
  --escrow_id $(id)

add_allowed_token:
	stellar contract invoke \
  	--id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A \
  	--source-account maycon \
  	-- add_allowed_token \
  	--token CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC

get_allowed_tokens:
	stellar contract invoke --id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A --source-account maycon -- get_allowed_tokens

get_nonce:
	stellar contract invoke \
	--id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A \
	--source-account maycon \
	-- get_nonce \
	--user maycon

release_payment:
	stellar contract invoke \
  	--id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A \
  	--source-account maycon \
  	-- release_payment \
  	--escrow_id $(id) \
  	--seller GBBIVZN5N7EMYMQHZL4ME64GWDM5REJDLFBDET7KLIIA6GQRQVJ2IQWE \
  	--nonce $(nonce)


request_refund:
	stellar contract invoke \
  --id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A \
  --source-account maycon \
  -- request_refund \
  --escrow_id $(id) \
  --buyer maycon \
  --nonce $(nonce)


dispute_escrow:
	stellar contract invoke \
  --id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A \
  --source-account maycon \
  -- dispute_escrow \
  --escrow_id $(id) \
  --caller maycon \
  --nonce $(nonce)


prop_res:
	stellar contract invoke \
  --id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A \
  --source-account maycon \
  -- propose_resolution \
  --escrow_id $(id) \
  --caller maycon \
  --favor_buyer true \
  --nonce $(nonce)


res_disp:
	stellar contract invoke \
  --id CCC7I5HCCF7YTFFTXIHQ4DGZX4W6SODEMQF24CPJRS37KQNHGONYUL7A \
  --source-account maycon \
  -- resolve_dispute \
  --escrow_id $(id) \
  --caller maycon \
  --nonce $(nonce)
