// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {SP1Verifier} from "@sp1-contracts/SP1Verifier.sol";

/// @title ZKBenchmarks.
/// @author Succinct Labs
/// @notice This contract implements a simple example of verifying the proof of a computing a
///         zkbenchmarks number.
contract ZKBenchmarks is SP1Verifier {
    /// @notice The verification key for the zkbenchmarks program.
    bytes32 public zkbenchmarksProgramVkey;

    constructor(bytes32 _zkbenchmarksProgramVkey) {
        zkbenchmarksProgramVkey = _zkbenchmarksProgramVkey;
    }

    /// @notice The entrypoint for verifying the proof of a zkbenchmarks number.
    /// @param proof The encoded proof.
    /// @param publicValues The encoded public values.
    function verifyZKBenchmarksProof(
        bytes memory proof,
        bytes memory publicValues
    ) public view returns (uint32, uint32, uint32) {
        this.verifyProof(zkbenchmarksProgramVkey, publicValues, proof);
        (uint32 n, uint32 a, uint32 b) = abi.decode(
            publicValues,
            (uint32, uint32, uint32)
        );
        return (n, a, b);
    }
}
