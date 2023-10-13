import {Policy, PolicyRegisterRequest} from "../idl/vault";
import {expect} from "chai";
import {principalToAddress} from "ictool";
import {Principal} from "@dfinity/principal";

describe("PPPPPP", () => {

    it("register123123", async function () {
     let a =   principalToAddress(Principal.fromText("gknwf-6bime-skguh-ck2jn-bxqam-otcti-vb2z3-thrwy-rguna-mhhjh-gae") as any, Array(32).fill(1))
        console.log(a)
        "deaba58d6ad04bc6122a379cb6c0e89a33b24ae77551b00b7d9969d56fc1a861"
    });

})