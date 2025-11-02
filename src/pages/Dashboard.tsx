import { useState, useEffect } from "react";
import { CarsList } from "../components/CarList";
import { CreateCarForm } from "../components/CreateCarForm";
import StellarExpertLink from "../components/StellarExpertLink";
import useModal from "../hooks/useModal";
import { ICar } from "../interfaces/car";
import { CarStatus } from "../interfaces/car-status";
import { IRentACarContract } from "../interfaces/contract";
import { CreateCar } from "../interfaces/create-car";
import { UserRole } from "../interfaces/user-role";
import { useStellarAccounts } from "../providers/StellarAccountProvider";
import { stellarService } from "../services/stellar.service";
import { walletService } from "../services/wallet.service";
import { ONE_XLM_IN_STROOPS } from "../utils/xlm-in-stroops";

export default function Dashboard() {
  const { hashId, cars, walletAddress, setCars, setHashId, selectedRole } =
    useStellarAccounts();
  const { showModal, openModal, closeModal } = useModal();

  // Admin panel state
  const [adminFee, setAdminFee] = useState<number>(0);
  const [accumulatedFees, setAccumulatedFees] = useState<number>(0);
  const [newFee, setNewFee] = useState<string>("");
  const [withdrawAmount, setWithdrawAmount] = useState<string>("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (selectedRole === UserRole.ADMIN) {
      void loadAdminData();
    }
  }, [walletAddress, selectedRole]);

  const loadAdminData = async () => {
    try {
      const contractClient =
        await stellarService.buildClient<IRentACarContract>(walletAddress);

      const fee = await contractClient.get_admin_fee();
      const accumulated = await contractClient.get_admin_accumulated_fees();

      setAdminFee(Number(fee));
      setAccumulatedFees(Number(accumulated));
    } catch (error) {
      console.error("Error loading admin data:", error);
    }
  };

  const handleCreateCar = async (formData: CreateCar) => {
    const { brand, model, color, passengers, pricePerDay, ac, ownerAddress } =
      formData;
    const contractClient =
      await stellarService.buildClient<IRentACarContract>(walletAddress);

    const addCarResult = await contractClient.add_car({
      owner: ownerAddress,
      price_per_day: pricePerDay * ONE_XLM_IN_STROOPS,
    });
    const xdr = addCarResult.toXDR();

    const signedTx = await walletService.signTransaction(xdr);
    const txResponse = await stellarService.submitTransaction(signedTx.signedTxXdr);

    const newCar: ICar = {
      brand,
      model,
      color,
      passengers,
      pricePerDay,
      ac,
      ownerAddress,
      status: CarStatus.AVAILABLE,
      availableToWithdraw: 0,
    };

    setCars((prevCars) => [...prevCars, newCar]);
    setHashId(txResponse.hash);
    closeModal();
  };

  const handleSetFee = async () => {
    if (!newFee || Number(newFee) < 0) {
      alert("Please enter a valid fee amount");
      return;
    }

    setLoading(true);
    try {
      const contractClient =
        await stellarService.buildClient<IRentACarContract>(walletAddress);

      const feeInStroops = Math.floor(Number(newFee) * ONE_XLM_IN_STROOPS);

      const result = await contractClient.set_admin_fee({
        admin: walletAddress,
        fee: feeInStroops,
      });
      const xdr = result.toXDR();

      const signedTx = await walletService.signTransaction(xdr);
      const txResponse = await stellarService.submitTransaction(signedTx.signedTxXdr);

      setHashId(txResponse.hash);
      setAdminFee(feeInStroops);
      setNewFee("");
      alert("Fee updated successfully!");
    } catch (error) {
      console.error("Error setting fee:", error);
      alert("Failed to set fee");
    } finally {
      setLoading(false);
    }
  };

  const handleWithdrawFees = async () => {
    if (!withdrawAmount || Number(withdrawAmount) <= 0) {
      alert("Please enter a valid amount");
      return;
    }

    const amountInStroops = Math.floor(Number(withdrawAmount) * ONE_XLM_IN_STROOPS);

    if (amountInStroops > accumulatedFees) {
      alert("Insufficient accumulated fees");
      return;
    }

    setLoading(true);
    try {
      const contractClient =
        await stellarService.buildClient<IRentACarContract>(walletAddress);

      const result = await contractClient.withdraw_admin_fees({
        admin: walletAddress,
        amount: amountInStroops,
      });
      const xdr = result.toXDR();

      const signedTx = await walletService.signTransaction(xdr);
      const txResponse = await stellarService.submitTransaction(signedTx.signedTxXdr);

      setHashId(txResponse.hash);
      setAccumulatedFees(accumulatedFees - amountInStroops);
      setWithdrawAmount("");
      alert("Fees withdrawn successfully!");
    } catch (error) {
      console.error("Error withdrawing fees:", error);
      alert("Failed to withdraw fees");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold" data-test="dashboard-title">
          Cars Catalog
        </h1>
        {selectedRole === UserRole.ADMIN && (
          <button
            onClick={openModal}
            className="group px-6 py-3 bg-indigo-600 text-white font-semibold rounded-xl shadow-lg hover:bg-indigo-700 hover:shadow-xl disabled:bg-slate-300 disabled:cursor-not-allowed transition-all duration-200 transform hover:scale-105 disabled:transform-none cursor-pointer"
          >
            <span className="flex items-center gap-2">Add Car</span>
          </button>
        )}
      </div>

      {/* Admin Panel */}
      {selectedRole === UserRole.ADMIN && (
        <div className="bg-white shadow-md rounded-lg p-6 mb-6">
          <h2 className="text-xl font-bold mb-4 text-gray-800">Admin Panel</h2>

          {/* Current Stats */}
          <div className="grid grid-cols-2 gap-4 mb-6">
            <div className="bg-blue-50 p-4 rounded-lg">
              <p className="text-sm text-gray-600">Current Fee per Rental</p>
              <p className="text-2xl font-bold text-blue-600">
                {(adminFee / ONE_XLM_IN_STROOPS).toFixed(2)} XLM
              </p>
            </div>
            <div className="bg-green-50 p-4 rounded-lg">
              <p className="text-sm text-gray-600">Accumulated Fees</p>
              <p className="text-2xl font-bold text-green-600">
                {(accumulatedFees / ONE_XLM_IN_STROOPS).toFixed(2)} XLM
              </p>
            </div>
          </div>

          {/* Set Fee Section */}
          <div className="mb-6">
            <h3 className="text-lg font-semibold mb-3 text-gray-700">
              Set Rental Commission
            </h3>
            <div className="flex gap-3">
              <input
                type="number"
                value={newFee}
                onChange={(e) => setNewFee(e.target.value)}
                placeholder="Enter fee in XLM"
                min="0"
                step="0.01"
                className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                disabled={loading}
              />
              <button
                onClick={() => void handleSetFee()}
                disabled={loading || !newFee}
                className="px-6 py-2 bg-blue-600 text-white rounded-lg font-semibold hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
              >
                {loading ? "Processing..." : "Set Fee"}
              </button>
            </div>
          </div>

          {/* Withdraw Fees Section */}
          <div>
            <h3 className="text-lg font-semibold mb-3 text-gray-700">
              Withdraw Accumulated Fees
            </h3>
            <div className="flex gap-3">
              <input
                type="number"
                value={withdrawAmount}
                onChange={(e) => setWithdrawAmount(e.target.value)}
                placeholder="Enter amount in XLM"
                min="0"
                step="0.01"
                max={(accumulatedFees / ONE_XLM_IN_STROOPS).toString()}
                className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
                disabled={loading || accumulatedFees === 0}
              />
              <button
                onClick={() => void handleWithdrawFees()}
                disabled={loading || !withdrawAmount || accumulatedFees === 0}
                className="px-6 py-2 bg-green-600 text-white rounded-lg font-semibold hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
              >
                {loading ? "Processing..." : "Withdraw"}
              </button>
            </div>
            {accumulatedFees === 0 && (
              <p className="text-sm text-gray-500 mt-2">
                No fees available to withdraw
              </p>
            )}
          </div>
        </div>
      )}

      {cars && <CarsList cars={cars} />}

      {showModal && (
        <CreateCarForm onCreateCar={handleCreateCar} onCancel={closeModal} />
      )}

      {hashId && <StellarExpertLink url={hashId} />}
    </div>
  );
}