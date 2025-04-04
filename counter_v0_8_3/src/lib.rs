#![cfg_attr(not(feature = "export-abi"), no_main)]

extern crate alloc;

use alloc::vec::Vec;
use stylus_sdk::prelude::*;
use stylus_sdk::alloy_primitives::U256;
use stylus_sdk::block;

sol_storage! {
    #[entrypoint]
    pub struct InterestCalculator {
        // 기본 상태 변수들
        uint256 principal;
        uint256 rate;
        uint256 period;
        uint256 compound;
        uint256 accumulated_interest;
        uint256 last_calculation;
        
        // 캐시 관련 변수들
        uint256 cached_rate;
        uint256 cached_period;
        uint256 cached_result;
        uint256 last_update;
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
        self.cached_rate.set(U256::ZERO);
        self.cached_period.set(U256::ZERO);
        self.cached_result.set(U256::ZERO);
        self.last_update.set(U256::ZERO);
        Ok(())
    }

    pub fn set_principal(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.principal.set(amount);
        self.invalidate_cache();
        Ok(())
    }

    pub fn set_rate(&mut self, new_rate: U256) -> Result<(), Vec<u8>> {
        if new_rate > U256::from(10000) {
            return Err("Rate too high".as_bytes().to_vec());
        }
        self.rate.set(new_rate);
        self.invalidate_cache();
        Ok(())
    }

    pub fn set_period(&mut self, new_period: U256) -> Result<(), Vec<u8>> {
        if new_period > U256::from(100) {
            return Err("Period too long".as_bytes().to_vec());
        }
        self.period.set(new_period);
        self.invalidate_cache();
        Ok(())
    }

    pub fn set_compound(&mut self, is_compound: U256) -> Result<(), Vec<u8>> {
        if is_compound > U256::ONE {
            return Err("Invalid compound flag".as_bytes().to_vec());
        }
        self.compound.set(is_compound);
        self.invalidate_cache();
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

    // 캐시 무효화 함수
    fn invalidate_cache(&mut self) {
        self.cached_rate.set(U256::ZERO);
        self.cached_period.set(U256::ZERO);
        self.cached_result.set(U256::ZERO);
        self.last_update.set(U256::ZERO);
    }

    // 캐시 유효성 검사
    fn is_cache_valid(&self) -> bool {
        let current_rate = self.rate.get();
        let current_period = self.period.get();
        let cached_rate = self.cached_rate.get();
        let cached_period = self.cached_period.get();
        
        cached_rate == current_rate && cached_period == current_period
    }

    // 최적화된 복리 이자 계산 함수
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
            // 캐시된 값이 있는지 확인
            if self.is_cache_valid() {
                // 캐시된 결과 사용
                interest = self.cached_result.get();
            } else {
                // 복리 이자 계산: A = P(1 + r)^t
                // 여기서 r은 소수점으로 변환 (예: 550 -> 0.055)
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
                        .ok_or_else(|| "Base division overflow".as_bytes().to_vec())?;
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
                
                // 결과 캐싱
                self.cached_rate.set(rate);
                self.cached_period.set(period);
                self.cached_result.set(interest);
                self.last_update.set(U256::from(block::timestamp()));
            }
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