#![cfg_attr(not(feature = "export-abi"), no_main)]

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::U256;
use stylus_sdk::prelude::*;
use stylus_sdk::block;

#[derive(Debug)]
pub enum Error {
    Overflow,
}

sol_storage! {
    #[entrypoint]
    pub struct InterestCalculator {
        // 원금
        uint256 principal;
        // 이자율 (기본값: 5.5% = 550)
        uint256 rate;
        // 기간 (년)
        uint256 period;
        // 복리 계산 여부 (1: 복리, 0: 단리)
        uint256 compound;
        // 누적 이자
        uint256 accumulated_interest;
        // 마지막 계산 시간
        uint256 last_calculation;
    }
}

#[public]
impl InterestCalculator {
    pub fn initialize(&mut self) -> Result<(), Vec<u8>> {
        self.principal.set(U256::ZERO);
        self.rate.set(U256::from(550)); // 기본 5.5%
        self.period.set(U256::ONE);
        self.compound.set(U256::ONE); // 기본적으로 복리
        self.accumulated_interest.set(U256::ZERO);
        self.last_calculation.set(U256::ZERO);
        Ok(())
    }

    pub fn set_principal(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.principal.set(amount);
        Ok(())
    }

    pub fn set_rate(&mut self, new_rate: U256) -> Result<(), Vec<u8>> {
        if new_rate > U256::from(10000) {
            return Err("Rate too high".as_bytes().to_vec());
        }
        // 이자율 변경 시 이전 이자를 누적하고, 새로운 이자율로 이자를 계산
        let current_interest = self.accumulated_interest.get();
        let current_principal = self.principal.get();
        self.principal.set(current_principal + current_interest);
        self.accumulated_interest.set(U256::ZERO);
        self.rate.set(new_rate);
        Ok(())
    }

    pub fn set_period(&mut self, new_period: U256) -> Result<(), Vec<u8>> {
        if new_period > U256::from(100) {
            return Err("Period too long".as_bytes().to_vec());
        }
        self.period.set(new_period);
        Ok(())
    }

    pub fn set_compound(&mut self, is_compound: U256) -> Result<(), Vec<u8>> {
        if is_compound > U256::ONE {
            return Err("Invalid compound flag".as_bytes().to_vec());
        }
        self.compound.set(is_compound);
        Ok(())
    }

    pub fn get_principal(&self) -> U256 {
        self.principal.get()
    }

    pub fn get_rate(&self) -> U256 {
        self.rate.get()
    }

    pub fn get_period(&self) -> U256 {
        self.period.get()
    }

    pub fn get_accumulated_interest(&self) -> U256 {
        self.accumulated_interest.get()
    }

    pub fn calculate_interest(&mut self) -> Result<U256, Vec<u8>> {
        let principal = self.principal.get();
        let rate = self.rate.get();
        let period = self.period.get();
        let compound = self.compound.get();
        
        if principal == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let mut interest: U256;
        if compound == U256::ONE {
            // 복리 이자 계산: A = P(1 + r)^t
            // 여기서 r은 소수점으로 변환 (예: 1500 -> 0.15)
            let rate_decimal = rate
                .checked_mul(U256::from(1_000_000))
                .ok_or_else(|| "Rate multiplication overflow".as_bytes().to_vec())?;
            let rate_decimal = rate_decimal
                .checked_div(U256::from(100_000_000))
                .ok_or_else(|| "Rate division overflow".as_bytes().to_vec())?;

            // (1 + r)^t 계산
            let mut base = U256::from(1_000_000)
                .checked_add(rate_decimal)
                .ok_or_else(|| "Base addition overflow".as_bytes().to_vec())?;
            let mut result = U256::from(1_000_000);
            
            // 지수 계산 (최적화된 방식)
            let period_value = period.to_string().parse::<u32>()
                .map_err(|_| "Invalid period value".as_bytes().to_vec())?;
            
            // 이진 지수화 알고리즘 사용
            let mut power = period_value;
            while power > 0 {
                if power & 1 == 1 {
                    result = result
                        .checked_mul(base)
                        .ok_or_else(|| "Exponentiation overflow".as_bytes().to_vec())?;
                    result = result
                        .checked_div(U256::from(1_000_000))
                        .ok_or_else(|| "Division overflow".as_bytes().to_vec())?;
                }
                base = base
                    .checked_mul(base)
                    .ok_or_else(|| "Base multiplication overflow".as_bytes().to_vec())?;
                base = base
                    .checked_div(U256::from(1_000_000))
                    .ok_or_else(|| "Division overflow".as_bytes().to_vec())?;
                power >>= 1;
            }
            
            // 최종 이자 계산
            interest = principal
                .checked_mul(result)
                .ok_or_else(|| "Final multiplication overflow".as_bytes().to_vec())?;
            interest = interest
                .checked_div(U256::from(1_000_000))
                .ok_or_else(|| "Final division overflow".as_bytes().to_vec())?;
            interest = interest
                .checked_sub(principal)
                .ok_or_else(|| "Final subtraction overflow".as_bytes().to_vec())?;
        } else {
            // 단리 이자 계산: I = P * r * t
            interest = principal
                .checked_mul(rate)
                .ok_or_else(|| "Simple interest multiplication overflow".as_bytes().to_vec())?;
            interest = interest
                .checked_mul(period)
                .ok_or_else(|| "Simple interest period multiplication overflow".as_bytes().to_vec())?;
            interest = interest
                .checked_div(U256::from(100_000_000))
                .ok_or_else(|| "Simple interest division overflow".as_bytes().to_vec())?;
        }

        // 누적 이자 업데이트
        let new_accumulated = self.accumulated_interest.get()
            .checked_add(interest)
            .ok_or_else(|| "Accumulated interest addition overflow".as_bytes().to_vec())?;
        self.accumulated_interest.set(new_accumulated);
        
        // 마지막 계산 시간 업데이트
        self.last_calculation.set(U256::from(block::timestamp()));

        Ok(interest)
    }
} 