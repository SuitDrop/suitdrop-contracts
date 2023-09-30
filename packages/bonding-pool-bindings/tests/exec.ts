import { default as setup, calc_spot_price } from "../pkg/cw-bonding-pool";

type Uint128 = string;

type CurveType =
  | {
      constant: {
        scale: number;
        value: Uint128;
      };
    }
  | {
      linear: {
        scale: number;
        slope: Uint128;
      };
    }
  | {
      square_root: {
        scale: number;
        slope: Uint128;
      };
    }
  | {
      square_root_cubed: {
        scale: number;
        slope: Uint128;
      };
    }
  | {
      cube_root_squared: {
        scale: number;
        slope: Uint128;
      };
    };

interface CurveState {
  decimals: DecimalPlaces;
  reserve: Uint128;
  reserve_denom: string;
  supply: Uint128;
  supply_denom: string;
}
interface DecimalPlaces {
  reserve: number;
  supply: number;
}

interface CwBondingInstantiateMsg {
  curve_type: CurveType;
  max_supply: Uint128;
  reserve_decimals: number;
  reserve_denom: string;
  supply_decimals: number;
  supply_subdenom: string;
  test_mode?: boolean | null;
}

await setup();

const out: string = calc_spot_price({
  quote_asset_denom: "usdc",
  base_asset_denom: "btc",
  curve_state: {
    decimals: {
      reserve: 6,
      supply: 8,
    },
    reserve: "0",
    reserve_denom: "btc",
    supply: "1000",
    supply_denom: "usdc",
  },
  curve_type: {
    cube_root_squared: {
      scale: 1,
      slope: "1",
    },
  },
});

console.log(out);
