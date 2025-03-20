pub const ABI: &str  = r#"[
    {
      "inputs": [
        {
          "internalType": "string",
          "name": "name_",
          "type": "string"
        },
        {
          "internalType": "string",
          "name": "symbol_",
          "type": "string"
        },
        {
          "internalType": "uint8",
          "name": "decimals_",
          "type": "uint8"
        },
        {
          "internalType": "uint256",
          "name": "totalSupply",
          "type": "uint256"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "constructor"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "spender",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "allowance",
          "type": "uint256"
        },
        {
          "internalType": "uint256",
          "name": "needed",
          "type": "uint256"
        }
      ],
      "name": "ERC20InsufficientAllowance",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "sender",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "balance",
          "type": "uint256"
        },
        {
          "internalType": "uint256",
          "name": "needed",
          "type": "uint256"
        }
      ],
      "name": "ERC20InsufficientBalance",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "approver",
          "type": "address"
        }
      ],
      "name": "ERC20InvalidApprover",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "receiver",
          "type": "address"
        }
      ],
      "name": "ERC20InvalidReceiver",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "sender",
          "type": "address"
        }
      ],
      "name": "ERC20InvalidSender",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "spender",
          "type": "address"
        }
      ],
      "name": "ERC20InvalidSpender",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "owner",
          "type": "address"
        }
      ],
      "name": "OwnableInvalidOwner",
      "type": "error"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "account",
          "type": "address"
        }
      ],
      "name": "OwnableUnauthorizedAccount",
      "type": "error"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": true,
          "internalType": "address",
          "name": "owner",
          "type": "address"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "spender",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "Approval",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": true,
          "internalType": "address",
          "name": "previousOwner",
          "type": "address"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "newOwner",
          "type": "address"
        }
      ],
      "name": "OwnershipTransferred",
      "type": "event"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": true,
          "internalType": "address",
          "name": "from",
          "type": "address"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "to",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "Transfer",
      "type": "event"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "owner",
          "type": "address"
        },
        {
          "internalType": "address",
          "name": "spender",
          "type": "address"
        }
      ],
      "name": "allowance",
      "outputs": [
        {
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "spender",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "approve",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "account",
          "type": "address"
        }
      ],
      "name": "balanceOf",
      "outputs": [
        {
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "owner",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "amount",
          "type": "uint256"
        }
      ],
      "name": "burn",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "decimals",
      "outputs": [
        {
          "internalType": "uint8",
          "name": "",
          "type": "uint8"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "name",
      "outputs": [
        {
          "internalType": "string",
          "name": "",
          "type": "string"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "owner",
      "outputs": [
        {
          "internalType": "address",
          "name": "",
          "type": "address"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "renounceOwnership",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "symbol",
      "outputs": [
        {
          "internalType": "string",
          "name": "",
          "type": "string"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [],
      "name": "totalSupply",
      "outputs": [
        {
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        }
      ],
      "stateMutability": "view",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "to",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "transfer",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "from",
          "type": "address"
        },
        {
          "internalType": "address",
          "name": "to",
          "type": "address"
        },
        {
          "internalType": "uint256",
          "name": "value",
          "type": "uint256"
        }
      ],
      "name": "transferFrom",
      "outputs": [
        {
          "internalType": "bool",
          "name": "",
          "type": "bool"
        }
      ],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "newOwner",
          "type": "address"
        }
      ],
      "name": "transferOwnership",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "string",
          "name": "name_",
          "type": "string"
        }
      ],
      "name": "updateName",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "inputs": [
        {
          "internalType": "string",
          "name": "symbol_",
          "type": "string"
        }
      ],
      "name": "updateSymbol",
      "outputs": [],
      "stateMutability": "nonpayable",
      "type": "function"
    }
]"#;

pub const  BYTECODE: &str = "0x60806040523480156200001157600080fd5b50604051620010a2380380620010a2833981016040819052620000349162000367565b338484600362000045838262000481565b50600462000054828262000481565b5050506001600160a01b0381166200008757604051631e4fbdf760e01b8152600060048201526024015b60405180910390fd5b6200009281620000df565b506005805460ff60a01b1916600160a01b60ff8516021790556007620000b9848262000481565b506006620000c8858262000481565b50620000d5338262000131565b5050505062000575565b600580546001600160a01b038381166001600160a01b0319831681179093556040519116919082907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a35050565b6001600160a01b0382166200015d5760405163ec442f0560e01b8152600060048201526024016200007e565b6200016b600083836200016f565b5050565b6001600160a01b0383166200019e5780600260008282546200019291906200054d565b90915550620002129050565b6001600160a01b03831660009081526020819052604090205481811015620001f35760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016200007e565b6001600160a01b03841660009081526020819052604090209082900390555b6001600160a01b03821662000230576002805482900390556200024f565b6001600160a01b03821660009081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040516200029591815260200190565b60405180910390a3505050565b634e487b7160e01b600052604160045260246000fd5b600082601f830112620002ca57600080fd5b81516001600160401b0380821115620002e757620002e7620002a2565b604051601f8301601f19908116603f01168101908282118183101715620003125762000312620002a2565b816040528381526020925086838588010111156200032f57600080fd5b600091505b8382101562000353578582018301518183018401529082019062000334565b600093810190920192909252949350505050565b600080600080608085870312156200037e57600080fd5b84516001600160401b03808211156200039657600080fd5b620003a488838901620002b8565b95506020870151915080821115620003bb57600080fd5b50620003ca87828801620002b8565b935050604085015160ff81168114620003e257600080fd5b6060959095015193969295505050565b600181811c908216806200040757607f821691505b6020821081036200042857634e487b7160e01b600052602260045260246000fd5b50919050565b601f8211156200047c57600081815260208120601f850160051c81016020861015620004575750805b601f850160051c820191505b81811015620004785782815560010162000463565b5050505b505050565b81516001600160401b038111156200049d576200049d620002a2565b620004b581620004ae8454620003f2565b846200042e565b602080601f831160018114620004ed5760008415620004d45750858301515b600019600386901b1c1916600185901b17855562000478565b600085815260208120601f198616915b828110156200051e57888601518255948401946001909101908401620004fd565b50858210156200053d5787850151600019600388901b60f8161c191681555b5050505050600190811b01905550565b808201808211156200056f57634e487b7160e01b600052601160045260246000fd5b92915050565b610b1d80620005856000396000f3fe608060405234801561001057600080fd5b50600436106100f55760003560e01c8063715018a6116100975780639dc29fac116100665780639dc29fac146101fb578063a9059cbb1461020e578063dd62ed3e14610221578063f2fde38b1461025a57600080fd5b8063715018a6146101bd57806384da92a7146101c55780638da5cb5b146101d857806395d89b41146101f357600080fd5b806323b872dd116100d357806323b872dd1461014d578063313ce56714610160578063537f53121461017f57806370a082311461019457600080fd5b806306fdde03146100fa578063095ea7b31461011857806318160ddd1461013b575b600080fd5b61010261026d565b60405161010f9190610792565b60405180910390f35b61012b6101263660046107fc565b6102ff565b604051901515815260200161010f565b6002545b60405190815260200161010f565b61012b61015b366004610826565b610319565b600554600160a01b900460ff1660405160ff909116815260200161010f565b61019261018d366004610878565b61033d565b005b61013f6101a2366004610929565b6001600160a01b031660009081526020819052604090205490565b610192610355565b6101926101d3366004610878565b610369565b6005546040516001600160a01b03909116815260200161010f565b61010261037d565b6101926102093660046107fc565b61038c565b61012b61021c3660046107fc565b61039e565b61013f61022f36600461094b565b6001600160a01b03918216600090815260016020908152604080832093909416825291909152205490565b610192610268366004610929565b6103ac565b60606006805461027c9061097e565b80601f01602080910402602001604051908101604052809291908181526020018280546102a89061097e565b80156102f55780601f106102ca576101008083540402835291602001916102f5565b820191906000526020600020905b8154815290600101906020018083116102d857829003601f168201915b5050505050905090565b60003361030d8185856103ef565b60019150505b92915050565b600033610327858285610401565b61033285858561047f565b506001949350505050565b6103456104de565b60076103518282610a06565b5050565b61035d6104de565b610367600061050b565b565b6103716104de565b60066103518282610a06565b60606007805461027c9061097e565b6103946104de565b610351828261055d565b60003361030d81858561047f565b6103b46104de565b6001600160a01b0381166103e357604051631e4fbdf760e01b8152600060048201526024015b60405180910390fd5b6103ec8161050b565b50565b6103fc8383836001610593565b505050565b6001600160a01b038381166000908152600160209081526040808320938616835292905220546000198114610479578181101561046a57604051637dc7a0d960e11b81526001600160a01b038416600482015260248101829052604481018390526064016103da565b61047984848484036000610593565b50505050565b6001600160a01b0383166104a957604051634b637e8f60e11b8152600060048201526024016103da565b6001600160a01b0382166104d35760405163ec442f0560e01b8152600060048201526024016103da565b6103fc838383610668565b6005546001600160a01b031633146103675760405163118cdaa760e01b81523360048201526024016103da565b600580546001600160a01b038381166001600160a01b0319831681179093556040519116919082907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a35050565b6001600160a01b03821661058757604051634b637e8f60e11b8152600060048201526024016103da565b61035182600083610668565b6001600160a01b0384166105bd5760405163e602df0560e01b8152600060048201526024016103da565b6001600160a01b0383166105e757604051634a1406b160e11b8152600060048201526024016103da565b6001600160a01b038085166000908152600160209081526040808320938716835292905220829055801561047957826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460405161065a91815260200190565b60405180910390a350505050565b6001600160a01b0383166106935780600260008282546106889190610ac6565b909155506107059050565b6001600160a01b038316600090815260208190526040902054818110156106e65760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016103da565b6001600160a01b03841660009081526020819052604090209082900390555b6001600160a01b03821661072157600280548290039055610740565b6001600160a01b03821660009081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161078591815260200190565b60405180910390a3505050565b600060208083528351808285015260005b818110156107bf578581018301518582016040015282016107a3565b506000604082860101526040601f19601f8301168501019250505092915050565b80356001600160a01b03811681146107f757600080fd5b919050565b6000806040838503121561080f57600080fd5b610818836107e0565b946020939093013593505050565b60008060006060848603121561083b57600080fd5b610844846107e0565b9250610852602085016107e0565b9150604084013590509250925092565b634e487b7160e01b600052604160045260246000fd5b60006020828403121561088a57600080fd5b813567ffffffffffffffff808211156108a257600080fd5b818401915084601f8301126108b657600080fd5b8135818111156108c8576108c8610862565b604051601f8201601f19908116603f011681019083821181831017156108f0576108f0610862565b8160405282815287602084870101111561090957600080fd5b826020860160208301376000928101602001929092525095945050505050565b60006020828403121561093b57600080fd5b610944826107e0565b9392505050565b6000806040838503121561095e57600080fd5b610967836107e0565b9150610975602084016107e0565b90509250929050565b600181811c9082168061099257607f821691505b6020821081036109b257634e487b7160e01b600052602260045260246000fd5b50919050565b601f8211156103fc57600081815260208120601f850160051c810160208610156109df5750805b601f850160051c820191505b818110156109fe578281556001016109eb565b505050505050565b815167ffffffffffffffff811115610a2057610a20610862565b610a3481610a2e845461097e565b846109b8565b602080601f831160018114610a695760008415610a515750858301515b600019600386901b1c1916600185901b1785556109fe565b600085815260208120601f198616915b82811015610a9857888601518255948401946001909101908401610a79565b5085821015610ab65787850151600019600388901b60f8161c191681555b5050505050600190811b01905550565b8082018082111561031357634e487b7160e01b600052601160045260246000fdfea264697066735822122093192268651a4297f75684c09aa729f153b9a0d8b2483fa8b51257d92baa620364736f6c63430008140033";