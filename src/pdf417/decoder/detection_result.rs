/*
 * Copyright 2013 ZXing authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::fmt::Display;

use crate::pdf417::pdf_417_common;

use super::{
    BarcodeMetadata, BoundingBox, Codeword, DetectionRXingResultColumn,
    DetectionRXingResultRowIndicatorColumn,
};

const ADJUST_ROW_NUMBER_SKIP: u32 = 2;

/**
 * @author Guenther Grau
 */
pub struct DetectionRXingResult<'a> {
    barcodeMetadata: BarcodeMetadata,
    detectionRXingResultColumns: Vec<Option<DetectionRXingResultColumn<'a>>>,
    boundingBox: BoundingBox<'a>,
    barcodeColumnCount: usize,
}

impl<'a> DetectionRXingResult<'_> {
    pub fn new(
        barcodeMetadata: BarcodeMetadata,
        boundingBox: BoundingBox<'a>,
    ) -> DetectionRXingResult<'a> {
        DetectionRXingResult {
            barcodeColumnCount: barcodeMetadata.getColumnCount() as usize,
            detectionRXingResultColumns: vec![None; barcodeMetadata.getColumnCount() as usize + 2],
            barcodeMetadata,
            boundingBox,
        }
        // this.barcodeMetadata = barcodeMetadata;
        // this.barcodeColumnCount = barcodeMetadata.getColumnCount();
        // this.boundingBox = boundingBox;
        // detectionRXingResultColumns = new DetectionRXingResultColumn[barcodeColumnCount + 2];
    }

    pub fn getDetectionRXingResultColumns(&mut self) -> &Vec<Option<DetectionRXingResultColumn>> {
        self.adjustIndicatorColumnRowNumbers(0);
        let pos = self.barcodeColumnCount + 1;
        self.adjustIndicatorColumnRowNumbers(pos);
        let mut unadjustedCodewordCount = pdf_417_common::MAX_CODEWORDS_IN_BARCODE;
        let mut previousUnadjustedCount;
        loop {
            previousUnadjustedCount = unadjustedCodewordCount;
            unadjustedCodewordCount = self.adjustRowNumbers();
            if !(unadjustedCodewordCount > 0 && unadjustedCodewordCount < previousUnadjustedCount) {
                break;
            }
        } //while (unadjustedCodewordCount > 0 && unadjustedCodewordCount < previousUnadjustedCount);
        &self.detectionRXingResultColumns
    }

    fn adjustIndicatorColumnRowNumbers(
        &mut self,
        pos: usize,
        // detectionRXingResultColumn: &mut Option<DetectionRXingResultColumn>,
    ) {
        if self.detectionRXingResultColumns[pos].is_some() {
            // if (detectionRXingResultColumn != null) {
            //   ((DetectionRXingResultRowIndicatorColumn) detectionRXingResultColumn)
            //       .adjustCompleteIndicatorColumnRowNumbers(barcodeMetadata);
            // }
            self.detectionRXingResultColumns[pos]
                .as_mut()
                .unwrap()
                .adjustCompleteIndicatorColumnRowNumbers(&self.barcodeMetadata);
        }
    }

    // TODO ensure that no detected codewords with unknown row number are left
    // we should be able to estimate the row height and use it as a hint for the row number
    // we should also fill the rows top to bottom and bottom to top
    /**
     * @return number of codewords which don't have a valid row number. Note that the count is not accurate as codewords
     * will be counted several times. It just serves as an indicator to see when we can stop adjusting row numbers
     */
    fn adjustRowNumbers(&mut self) -> u32 {
        let unadjustedCount = self.adjustRowNumbersByRow();
        if unadjustedCount == 0 {
            return 0;
        }
        for barcodeColumn in 1..(self.barcodeColumnCount + 1) {
            // for (int barcodeColumn = 1; barcodeColumn < barcodeColumnCount + 1; barcodeColumn++) {
            if self.detectionRXingResultColumns[barcodeColumn].is_some() {
                let codewords = self.detectionRXingResultColumns[barcodeColumn]
                    .as_ref()
                    .unwrap()
                    .getCodewords();
                for codewordsRow in 0..codewords.len() {
                    // for (int codewordsRow = 0; codewordsRow < codewords.length; codewordsRow++) {
                    if let Some(cw_row) = codewords[codewordsRow] {
                        if !cw_row.hasValidRowNumber() {
                            self.adjustRowNumbersWithCodewords(
                                barcodeColumn,
                                codewordsRow,
                                codewords,
                            );
                        }
                    } else {
                        continue;
                    }
                    // if (codewords[codewordsRow] == null) {
                    //   continue;
                    // }
                    // if (!codewords[codewordsRow].hasValidRowNumber()) {
                    //   self.adjustRowNumbers(barcodeColumn, codewordsRow, codewords);
                    // }
                }
            }
        }
        return unadjustedCount;
    }

    fn adjustRowNumbersByRow(&mut self) -> u32 {
        self.adjustRowNumbersFromBothRI();
        // TODO we should only do full row adjustments if row numbers of left and right row indicator column match.
        // Maybe it's even better to calculated the height (in codeword rows) and divide it by the number of barcode
        // rows. This, together with the LRI and RRI row numbers should allow us to get a good estimate where a row
        // number starts and ends.
        let unadjustedCount = self.adjustRowNumbersFromLRI();
        unadjustedCount + self.adjustRowNumbersFromRRI()
    }

    fn adjustRowNumbersFromBothRI(&mut self) {
        if self.detectionRXingResultColumns[0].is_some()
            && self.detectionRXingResultColumns[self.barcodeColumnCount as usize + 1].is_some()
        {
            // let LRIcodewords = self.detectionRXingResultColumns[0].as_ref().unwrap().getCodewords();
            // let RRIcodewords = self.detectionRXingResultColumns[self.barcodeColumnCount as usize + 1].as_ref().unwrap().getCodewords();
            for codewordsRow in 0..self.detectionRXingResultColumns[0]
                .as_ref()
                .unwrap()
                .getCodewords()
                .len()
            {
                // for (int codewordsRow = 0; codewordsRow < LRIcodewords.length; codewordsRow++) {
                if
                //let (Some(lricw), Some(rricw)) =
                self.detectionRXingResultColumns[0]
                    .as_ref()
                    .unwrap()
                    .getCodewords()[codewordsRow]
                    .is_some()
                    && self.detectionRXingResultColumns[self.barcodeColumnCount as usize + 1]
                        .as_ref()
                        .unwrap()
                        .getCodewords()[codewordsRow]
                        .is_some()
                {
                    if self.detectionRXingResultColumns[0]
                        .as_ref()
                        .unwrap()
                        .getCodewords()[codewordsRow]
                        .as_ref()
                        .unwrap()
                        .getRowNumber()
                        == self.detectionRXingResultColumns[self.barcodeColumnCount as usize + 1]
                            .as_ref()
                            .unwrap()
                            .getCodewords()[codewordsRow]
                            .as_ref()
                            .unwrap()
                            .getRowNumber()
                    {
                        // if (LRIcodewords[codewordsRow] != null &&
                        //     RRIcodewords[codewordsRow] != null &&
                        //     LRIcodewords[codewordsRow].getRowNumber() == RRIcodewords[codewordsRow].getRowNumber()) {
                        for barcodeColumn in 1..=self.barcodeColumnCount {
                            // for (int barcodeColumn = 1; barcodeColumn <= barcodeColumnCount; barcodeColumn++) {
                            if self.detectionRXingResultColumns[barcodeColumn].is_some()
                            //let Some(dc_col) =
                            //&mut self.detectionRXingResultColumns[barcodeColumn]
                            {
                                if self.detectionRXingResultColumns[barcodeColumn]
                                    .as_mut()
                                    .unwrap()
                                    .getCodewordsMut()[codewordsRow]
                                    .is_some()
                                {
                                    //let Some(codeword) = &mut self.detectionRXingResultColumns[barcodeColumn].as_mut().unwrap().getCodewordsMut()[codewordsRow] {
                                    let new_row_number = self.detectionRXingResultColumns[0]
                                        .as_ref()
                                        .unwrap()
                                        .getCodewords()[codewordsRow]
                                        .as_ref()
                                        .unwrap()
                                        .getRowNumber();
                                    self.detectionRXingResultColumns[barcodeColumn]
                                        .as_mut()
                                        .unwrap()
                                        .getCodewordsMut()[codewordsRow]
                                        .as_mut()
                                        .unwrap()
                                        .setRowNumber(new_row_number);
                                    if !self.detectionRXingResultColumns[barcodeColumn]
                                        .as_mut()
                                        .unwrap()
                                        .getCodewordsMut()[codewordsRow]
                                        .as_ref()
                                        .unwrap()
                                        .hasValidRowNumber()
                                    {
                                        // self.detectionRXingResultColumns[barcodeColumn].getCodewords()[codewordsRow] = None;
                                        self.detectionRXingResultColumns[barcodeColumn]
                                            .as_mut()
                                            .unwrap()
                                            .getCodewordsMut()[codewordsRow] = None;
                                    }
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                            // let codeword = self.detectionRXingResultColumns[barcodeColumn].getCodewords()[codewordsRow];
                            // if (codeword == null) {
                            //   continue;
                            // }
                        }
                    }
                }
            }
        }
        // if (detectionRXingResultColumns[0] == null || detectionRXingResultColumns[barcodeColumnCount + 1] == null) {
        //   return;
        // }
    }

    fn adjustRowNumbersFromRRI(&self) -> u32 {
        if let Some(col) = &self.detectionRXingResultColumns[self.barcodeColumnCount as usize + 1] {
            let mut unadjustedCount = 0;
            let codewords = col.getCodewords();
            for codewordsRow in 0..codewords.len() {
                // for (int codewordsRow = 0; codewordsRow < codewords.length; codewordsRow++) {
                if let Some(codeword_col) = codewords[codewordsRow] {
                    let rowIndicatorRowNumber = codeword_col.getRowNumber();
                    let mut invalidRowCounts = 0;
                    let mut barcodeColumn = self.barcodeColumnCount as usize + 1;
                    while barcodeColumn > 0 && invalidRowCounts < ADJUST_ROW_NUMBER_SKIP {
                        // for (int barcodeColumn = barcodeColumnCount + 1;
                        //      barcodeColumn > 0 && invalidRowCounts < ADJUST_ROW_NUMBER_SKIP;
                        //      barcodeColumn--) {
                        if let Some(bc_col) = &self.detectionRXingResultColumns[barcodeColumn] {
                            if let Some(codeword) = bc_col.getCodewords()[codewordsRow] {
                                invalidRowCounts = Self::adjustRowNumberIfValid(
                                    rowIndicatorRowNumber,
                                    invalidRowCounts,
                                    &mut Some(codeword),
                                );
                                if !codeword.hasValidRowNumber() {
                                    unadjustedCount += 1;
                                }
                            }
                        }
                        barcodeColumn -= 1;
                    }
                } else {
                    continue;
                }
            }
            unadjustedCount
        } else {
            0
        }
        // if (detectionRXingResultColumns[barcodeColumnCount + 1] == null) {
        //   return 0;
        // }
        // int unadjustedCount = 0;
        // Codeword[] codewords = detectionRXingResultColumns[barcodeColumnCount + 1].getCodewords();
        // for (int codewordsRow = 0; codewordsRow < codewords.length; codewordsRow++) {
        //   if (codewords[codewordsRow] == null) {
        //     continue;
        //   }
        //   int rowIndicatorRowNumber = codewords[codewordsRow].getRowNumber();
        //   int invalidRowCounts = 0;
        //   for (int barcodeColumn = barcodeColumnCount + 1;
        //        barcodeColumn > 0 && invalidRowCounts < ADJUST_ROW_NUMBER_SKIP;
        //        barcodeColumn--) {
        //     Codeword codeword = detectionRXingResultColumns[barcodeColumn].getCodewords()[codewordsRow];
        //     if (codeword != null) {
        //       invalidRowCounts = adjustRowNumberIfValid(rowIndicatorRowNumber, invalidRowCounts, codeword);
        //       if (!codeword.hasValidRowNumber()) {
        //         unadjustedCount++;
        //       }
        //     }
        //   }
        // }
        // return unadjustedCount;
    }

    fn adjustRowNumbersFromLRI(&self) -> u32 {
        if let Some(col) = &self.detectionRXingResultColumns[0] {
            let mut unadjustedCount = 0;
            let codewords = col.getCodewords();
            for codewordsRow in 0..codewords.len() {
                // for (int codewordsRow = 0; codewordsRow < codewords.length; codewordsRow++) {
                if let Some(codeword_in_row) = codewords[codewordsRow] {
                    let rowIndicatorRowNumber = codeword_in_row.getRowNumber();
                    let mut invalidRowCounts = 0;
                    let mut barcodeColumn = 1_usize;
                    while barcodeColumn < self.barcodeColumnCount as usize + 1
                        && invalidRowCounts < ADJUST_ROW_NUMBER_SKIP
                    {
                        // for (int barcodeColumn = 1;
                        //      barcodeColumn < barcodeColumnCount + 1 && invalidRowCounts < ADJUST_ROW_NUMBER_SKIP;
                        //      barcodeColumn++) {
                        if let Some(bc_column) = &self.detectionRXingResultColumns[barcodeColumn] {
                            if let Some(codeword) = bc_column.getCodewords()[codewordsRow] {
                                invalidRowCounts = Self::adjustRowNumberIfValid(
                                    rowIndicatorRowNumber,
                                    invalidRowCounts,
                                    &mut Some(codeword),
                                );
                                if !codeword.hasValidRowNumber() {
                                    unadjustedCount += 1;
                                }
                            }
                        }
                        // let codeword = self.detectionRXingResultColumns[barcodeColumn].getCodewords()[codewordsRow];
                        // if (codeword != null) {
                        //   invalidRowCounts = adjustRowNumberIfValid(rowIndicatorRowNumber, invalidRowCounts, codeword);
                        //   if (!codeword.hasValidRowNumber()) {
                        //     unadjustedCount+=1;
                        //   }
                        // }
                        barcodeColumn += 1;
                    }
                } else {
                    continue;
                }
                // if (codewords[codewordsRow] == null) {
                //   continue;
                // }
                // let rowIndicatorRowNumber = codewords[codewordsRow].getRowNumber();
                // let invalidRowCounts = 0;
                // for (int barcodeColumn = 1;
                //      barcodeColumn < barcodeColumnCount + 1 && invalidRowCounts < ADJUST_ROW_NUMBER_SKIP;
                //      barcodeColumn++) {
                //   Codeword codeword = detectionRXingResultColumns[barcodeColumn].getCodewords()[codewordsRow];
                //   if (codeword != null) {
                //     invalidRowCounts = adjustRowNumberIfValid(rowIndicatorRowNumber, invalidRowCounts, codeword);
                //     if (!codeword.hasValidRowNumber()) {
                //       unadjustedCount++;
                //     }
                //   }
                // }
            }
            unadjustedCount
        } else {
            0
        }

        // if (detectionRXingResultColumns[0] == null) {
        //   return 0;
        // }
        // int unadjustedCount = 0;
        // Codeword[] codewords = detectionRXingResultColumns[0].getCodewords();
        // for (int codewordsRow = 0; codewordsRow < codewords.length; codewordsRow++) {
        //   if (codewords[codewordsRow] == null) {
        //     continue;
        //   }
        //   int rowIndicatorRowNumber = codewords[codewordsRow].getRowNumber();
        //   int invalidRowCounts = 0;
        //   for (int barcodeColumn = 1;
        //        barcodeColumn < barcodeColumnCount + 1 && invalidRowCounts < ADJUST_ROW_NUMBER_SKIP;
        //        barcodeColumn++) {
        //     Codeword codeword = detectionRXingResultColumns[barcodeColumn].getCodewords()[codewordsRow];
        //     if (codeword != null) {
        //       invalidRowCounts = adjustRowNumberIfValid(rowIndicatorRowNumber, invalidRowCounts, codeword);
        //       if (!codeword.hasValidRowNumber()) {
        //         unadjustedCount++;
        //       }
        //     }
        //   }
        // }
        // return unadjustedCount;
    }

    fn adjustRowNumberIfValid(
        rowIndicatorRowNumber: i32,
        mut invalidRowCounts: u32,
        codeword: &mut Option<Codeword>,
    ) -> u32 {
        if let Some(codeword) = codeword {
            if !codeword.hasValidRowNumber() {
                if codeword.isValidRowNumber(rowIndicatorRowNumber) {
                    codeword.setRowNumber(rowIndicatorRowNumber);
                    invalidRowCounts = 0;
                } else {
                    invalidRowCounts += 1;
                }
            }
            invalidRowCounts
        } else {
            invalidRowCounts
        }
        // if (codeword == null) {
        //   return invalidRowCounts;
        // }
        // if (!codeword.hasValidRowNumber()) {
        //   if (codeword.isValidRowNumber(rowIndicatorRowNumber)) {
        //     codeword.setRowNumber(rowIndicatorRowNumber);
        //     invalidRowCounts = 0;
        //   } else {
        //     invalidRowCounts+=1;
        //   }
        // }
        // return invalidRowCounts;
    }

    fn adjustRowNumbersWithCodewords(
        &self,
        barcodeColumn: usize,
        codewordsRow: usize,
        codewords: &[Option<Codeword>],
    ) {
        let mut codeword = codewords[codewordsRow];
        let previousColumnCodewords = self.detectionRXingResultColumns[barcodeColumn - 1]
            .as_ref()
            .unwrap()
            .getCodewords();
        let mut nextColumnCodewords = previousColumnCodewords;
        if self.detectionRXingResultColumns[barcodeColumn + 1].is_some() {
            nextColumnCodewords = self.detectionRXingResultColumns[barcodeColumn + 1]
                .as_ref()
                .unwrap()
                .getCodewords(); //col.getCodewords();
        }
        // if (self.detectionRXingResultColumns[barcodeColumn + 1] != null) {
        //   nextColumnCodewords = self.detectionRXingResultColumns[barcodeColumn + 1].getCodewords();
        // }

        let mut otherCodewords = [None; 14]; // new Codeword[14];

        otherCodewords[2] = previousColumnCodewords[codewordsRow];
        otherCodewords[3] = nextColumnCodewords[codewordsRow];

        if codewordsRow > 0 {
            otherCodewords[0] = codewords[codewordsRow - 1];
            otherCodewords[4] = previousColumnCodewords[codewordsRow - 1];
            otherCodewords[5] = nextColumnCodewords[codewordsRow - 1];
        }
        if codewordsRow > 1 {
            otherCodewords[8] = codewords[codewordsRow - 2];
            otherCodewords[10] = previousColumnCodewords[codewordsRow - 2];
            otherCodewords[11] = nextColumnCodewords[codewordsRow - 2];
        }
        if codewordsRow < codewords.len() - 1 {
            otherCodewords[1] = codewords[codewordsRow + 1];
            otherCodewords[6] = previousColumnCodewords[codewordsRow + 1];
            otherCodewords[7] = nextColumnCodewords[codewordsRow + 1];
        }
        if codewordsRow < codewords.len() - 2 {
            otherCodewords[9] = codewords[codewordsRow + 2];
            otherCodewords[12] = previousColumnCodewords[codewordsRow + 2];
            otherCodewords[13] = nextColumnCodewords[codewordsRow + 2];
        }
        for otherCodeword in otherCodewords {
            if Self::adjustRowNumber(codeword.as_mut().unwrap(), &otherCodeword) {
                return;
            }
        }
        // for (Codeword otherCodeword : otherCodewords) {
        //   if (adjustRowNumber(codeword, otherCodeword)) {
        //     return;
        //   }
        // }
    }

    /**
     * @return true, if row number was adjusted, false otherwise
     */
    fn adjustRowNumber(codeword: &mut Codeword, otherCodeword: &Option<Codeword>) -> bool {
        if let Some(otherCodeword) = otherCodeword {
            if otherCodeword.hasValidRowNumber()
                && otherCodeword.getBucket() == codeword.getBucket()
            {
                codeword.setRowNumber(otherCodeword.getRowNumber());
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn getBarcodeColumnCount(&self) -> usize {
        self.barcodeColumnCount
    }

    pub fn getBarcodeRowCount(&self) -> u32 {
        self.barcodeMetadata.getRowCount()
    }

    pub fn getBarcodeECLevel(&self) -> u32 {
        self.barcodeMetadata.getErrorCorrectionLevel()
    }

    // pub fn setBoundingBox(&'a mut self,  boundingBox:BoundingBox<'a>) {
    //   self.boundingBox = boundingBox;
    // }

    pub fn getBoundingBox(&self) -> &BoundingBox {
        &self.boundingBox
    }

    // pub fn setDetectionRXingResultColumn(&mut self,  barcodeColumn:usize,  detectionRXingResultColumn:Option<DetectionRXingResultColumn<'a>>) {
    //   self.detectionRXingResultColumns[barcodeColumn] = detectionRXingResultColumn;
    // }

    pub fn getDetectionRXingResultColumn(
        &self,
        barcodeColumn: usize,
    ) -> &Option<DetectionRXingResultColumn> {
        &self.detectionRXingResultColumns[barcodeColumn]
    }
}

impl Display for DetectionRXingResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

// @Override
//   public String toString() {
//     DetectionRXingResultColumn rowIndicatorColumn = detectionRXingResultColumns[0];
//     if (rowIndicatorColumn == null) {
//       rowIndicatorColumn = detectionRXingResultColumns[barcodeColumnCount + 1];
//     }
//     try (Formatter formatter = new Formatter()) {
//       for (int codewordsRow = 0; codewordsRow < rowIndicatorColumn.getCodewords().length; codewordsRow++) {
//         formatter.format("CW %3d:", codewordsRow);
//         for (int barcodeColumn = 0; barcodeColumn < barcodeColumnCount + 2; barcodeColumn++) {
//           if (detectionRXingResultColumns[barcodeColumn] == null) {
//             formatter.format("    |   ");
//             continue;
//           }
//           Codeword codeword = detectionRXingResultColumns[barcodeColumn].getCodewords()[codewordsRow];
//           if (codeword == null) {
//             formatter.format("    |   ");
//             continue;
//           }
//           formatter.format(" %3d|%3d", codeword.getRowNumber(), codeword.getValue());
//         }
//         formatter.format("%n");
//       }
//       return formatter.toString();
//     }
//   }
