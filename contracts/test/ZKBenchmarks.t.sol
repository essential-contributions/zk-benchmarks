// SPDX-License-Identifier: MIT
pragma solidity ^0.8.25;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {ZKBenchmarks} from "../src/ZKBenchmarks.sol";
import {SP1Verifier} from "@sp1-contracts/SP1Verifier.sol";

struct SP1ProofFixtureJson {
    uint32 a;
    uint32 b;
    uint32 n;
    bytes proof;
    bytes publicValues;
    bytes32 vkey;
}

contract ZKBenchmarksTest is Test {
    using stdJson for string;

    ZKBenchmarks public zkbenchmarks;

    function loadFixture() public view returns (SP1ProofFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/src/fixtures/fixture.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (SP1ProofFixtureJson));
    }

    function setUp() public {
        SP1ProofFixtureJson memory fixture = loadFixture();
        zkbenchmarks = new ZKBenchmarks(fixture.vkey);
    }

    function test_ValidZKBenchmarksProof() public view {
        SP1ProofFixtureJson memory fixture = loadFixture();
        (uint32 n, uint32 a, uint32 b) = zkbenchmarks.verifyZKBenchmarksProof(
            fixture.proof,
            fixture.publicValues
        );
        assert(n == fixture.n);
        assert(a == fixture.a);
        assert(b == fixture.b);
    }

    function testFail_InvalidZKBenchmarksProof() public view {
        SP1ProofFixtureJson memory fixture = loadFixture();

        // Create a fake proof.
        bytes memory fakeProof = new bytes(fixture.proof.length);

        zkbenchmarks.verifyZKBenchmarksProof(fakeProof, fixture.publicValues);
    }
}
