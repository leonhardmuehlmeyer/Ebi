#[cfg(test)]
mod tests {
    use std::{fs, rc::Rc};

    use crate::{activity_key::{ActivityKey, ActivityKeyTranslator}, deterministic_semantics_for_stochastic_semantics::DeterministicStochasticSemantics, ebi_objects::{alignments::Move, event_log::EventLog, finite_language::FiniteLanguage, finite_stochastic_language::FiniteStochasticLanguage, finite_stochastic_language_semantics::FiniteStochasticLanguageSemantics, labelled_petri_net::LabelledPetriNet, stochastic_deterministic_finite_automaton::StochasticDeterministicFiniteAutomaton, stochastic_labelled_petri_net::StochasticLabelledPetriNet}, ebi_traits::{ebi_trait_event_log::EbiTraitEventLog, ebi_trait_queriable_stochastic_language::EbiTraitQueriableStochasticLanguage, ebi_trait_stochastic_deterministic_semantics::StochasticDeterministicSemantics}, follower_semantics::FollowerSemantics, math::fraction::Fraction, medoid, medoid_non_stochastic, occurrences_miner, stochastic_labelled_petri_net_semantics::StochasticLabelledPetriNetSemantics, test, trace_probability, uniform_stochastic_miner::uniform_stochastic_miner, unit_earth_movers_stochastic_conformance::uemsc};

    use super::*;

    #[test]
    fn medoid() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba.slang").unwrap();
        let slang = fin.parse::<FiniteStochasticLanguage>().unwrap();
        let fout = fs::read_to_string("testfiles/ba.lang").unwrap();
        let medoid = medoid::medoid(&slang, &1).unwrap();
        assert_eq!(fout, medoid.to_string())
    }

    #[test]
    fn non_stochastic_medoid() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba.slang").unwrap();
        let slang = fin.parse::<FiniteStochasticLanguage>().unwrap();
        let fout = fs::read_to_string("testfiles/aa.lang").unwrap();
        let medoid = medoid_non_stochastic::medoid(&slang, &1).unwrap();
        assert_eq!(fout, medoid.to_string())
    }

    #[test]
    fn non_stochastic_clustering() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba.slang").unwrap();
        let slang = fin.parse::<FiniteStochasticLanguage>().unwrap();
        let cluster = medoid_non_stochastic::k_medoids_clustering(&slang, 1).unwrap();
        let fout = fs::read_to_string("testfiles/aa.lang").unwrap();
        assert_eq!(fout, cluster.to_string())
    }

    #[test]
    fn lpn_uniform() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba.lpn").unwrap();
        let lpn = fin.parse::<LabelledPetriNet>().unwrap();
        let slpn = uniform_stochastic_miner(Box::new(lpn));
        let fout = fs::read_to_string("testfiles/aa-ab-ba_uni.slpn").unwrap();
        assert_eq!(fout, slpn.to_string())
    }

    #[test]
    fn lpn_occurrence() {
        let fin1 = fs::read_to_string("testfiles/aa-ab-ba.lpn").unwrap();
        let lpn = fin1.parse::<LabelledPetriNet>().unwrap();
        let fin2 = fs::read_to_string("testfiles/aa-ab-ba.slang").unwrap();
        let slang = fin2.parse::<FiniteStochasticLanguage>().unwrap();
        let slpn = occurrences_miner::mine(Box::new(lpn), Box::new(slang));
        let fout = fs::read_to_string("testfiles/aa-ab-ba_occ.slpn").unwrap();
        assert_eq!(fout, slpn.to_string())
    }

    #[test]
    fn mode() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba_uni.slpn").unwrap();
        let slpn = fin.parse::<StochasticLabelledPetriNet>().unwrap();
        let semantics = StochasticLabelledPetriNet::get_deterministic_semantics(Rc::new(slpn));
        let slang = semantics.analyse_most_likely_traces(&1).unwrap();
        let fout = fs::read_to_string("testfiles/ba.slang").unwrap();
        assert_eq!(fout, slang.to_string())
    }

    #[test]
    fn fraction_exact() {
        let mut zero = Fraction::one().one_minus();

        assert!(zero.is_zero());
    }

    //test disabled due to inconsistent output order (which is within spec)
    // #[test]
    // fn slang_to_sdfa() {
    //     let fin = fs::read_to_string("testfiles/aa-ab-ba.slang").unwrap();
    //     let slang = fin.parse::<FiniteStochasticLanguage>().unwrap();
    //     let sdfa = slang.to_stochastic_deterministic_finite_automaton();
    //     let fout = fs::read_to_string("testfiles/aa-ab-ba.sdfa").unwrap();
    //     assert_eq!(fout, sdfa.to_string())
    // }

    #[test]
    fn sdfa_minprob_one() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba.sdfa").unwrap();
        let sdfa = fin.parse::<StochasticDeterministicFiniteAutomaton>().unwrap();
        let semantics = StochasticDeterministicFiniteAutomaton::get_deterministic_semantics(Rc::new(sdfa)).unwrap();
        assert!(semantics.analyse_minimum_probability(&Fraction::one()).is_err());
    }

    #[test]
    fn sdfa_minprob_zero() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba.sdfa").unwrap();
        let sdfa = fin.parse::<StochasticDeterministicFiniteAutomaton>().unwrap();
        let semantics = StochasticDeterministicFiniteAutomaton::get_deterministic_semantics(Rc::new(sdfa)).unwrap();

        let slang2 = semantics.analyse_minimum_probability(&Fraction::zero()).unwrap();

        let should_string = fs::read_to_string("testfiles/aa-ab-ba.slang").unwrap();
        let should_slang = should_string.parse::<FiniteStochasticLanguage>().unwrap();
        assert_eq!(should_slang, slang2)
    }

    #[test]
    fn sdfa_minprob_zero_loop() {
        let fin = fs::read_to_string("testfiles/a-loop.sdfa").unwrap();
        let sdfa = fin.parse::<StochasticDeterministicFiniteAutomaton>().unwrap();
        let semantics = StochasticDeterministicFiniteAutomaton::get_deterministic_semantics(Rc::new(sdfa)).unwrap();

        assert!(semantics.analyse_minimum_probability(&Fraction::zero()).is_err());
    }

    #[test]
    fn slang_minprob_one() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba.slang").unwrap();
        let slang = fin.parse::<FiniteStochasticLanguage>().unwrap();
        let semantics = slang.to_deterministic_stochastic_semantics().unwrap();
        
        //should error
        assert!(semantics.analyse_minimum_probability(&Fraction::one()).is_err());
    }

    #[test]
    fn slang_minprob_zero() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba.slang").unwrap();
        let slang = fin.parse::<FiniteStochasticLanguage>().unwrap();
        let semantics = slang.to_deterministic_stochastic_semantics().unwrap();
        
        //should return the same
        let slang2 = semantics.analyse_minimum_probability(&Fraction::zero()).unwrap();

        assert_eq!(slang, slang2)
    }

    #[test]
    fn cla_test() {
        let fin = fs::read_to_string("testfiles/a-b.xes").unwrap();
        let event_log: Box<dyn EbiTraitEventLog> = Box::new(fin.parse::<EventLog>().unwrap());

        let (x, sustain) = test::log_categorical_attribute(&event_log, 500, &"attribute".to_string(), &Fraction::from((1, 20))).unwrap();
        assert!(sustain) //The hypothesis should be rejected if we consider the meaning of things, however, as we have only two traces, it will be sustained.
    }

    #[test]
    fn sdfa_invalid() {
        let fin = fs::read_to_string("testfiles/a-b-c-livelock_invalid.sdfa").unwrap();
        let mut err = fin.parse::<StochasticDeterministicFiniteAutomaton>();
        assert!(err.is_err());
    }

    #[test]
    fn sdfa_trace_prob_livelock() {
        let fin = fs::read_to_string("testfiles/a-b-c-livelock.sdfa").unwrap();
        let mut sdfa = fin.parse::<StochasticDeterministicFiniteAutomaton>().unwrap();

        //a
        {
            let strace = vec!["a".to_string()];
            let trace = sdfa.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = sdfa.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "2/5".parse::<Fraction>().unwrap());
        }

        //b
        {
            let strace = vec!["b".to_string()];
            let trace = sdfa.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = sdfa.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "1/5".parse::<Fraction>().unwrap());
        }

        //c (part of livelock trace)
        {
            let strace = vec!["c".to_string()];
            let trace = sdfa.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = sdfa.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "0".parse::<Fraction>().unwrap());
        }

        //d (non-existing label)
        {
            let strace = vec!["d".to_string()];
            let trace = sdfa.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = sdfa.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "0".parse::<Fraction>().unwrap());
        }
    }

    #[test]
    fn slpn_trace_prob_livelock() {
        let fin = fs::read_to_string("testfiles/a-b-c-livelock.slpn").unwrap();
        let mut slpn = fin.parse::<StochasticLabelledPetriNet>().unwrap();

        //a
        {
            let strace = vec!["a".to_string()];
            let trace = slpn.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = slpn.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "2/5".parse::<Fraction>().unwrap());
        }

        //b
        {
            let strace = vec!["b".to_string()];
            let trace = slpn.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = slpn.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "1/5".parse::<Fraction>().unwrap());
        }

        //c (part of livelock trace)
        {
            let strace = vec!["c".to_string()];
            let trace = slpn.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = slpn.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "0".parse::<Fraction>().unwrap());
        }

        //d (non-existing label)
        {
            let strace = vec!["d".to_string()];
            let trace = slpn.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = slpn.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "0".parse::<Fraction>().unwrap());
        }
    }

    #[test]
    fn slpn_trace_prob_tau_livelock() {
        let fin = fs::read_to_string("testfiles/a-b-tau-livelock.slpn").unwrap();
        let mut slpn = fin.parse::<StochasticLabelledPetriNet>().unwrap();

        //a
        {
            let strace = vec!["a".to_string()];
            let trace = slpn.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = slpn.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "2/5".parse::<Fraction>().unwrap());
        }

        //b
        {
            let strace = vec!["b".to_string()];
            let trace = slpn.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = slpn.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "1/5".parse::<Fraction>().unwrap());
        }

        //empty trace (part of livelock trace)
        {
            let strace = vec![];
            let trace = slpn.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = slpn.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "0".parse::<Fraction>().unwrap());
        }

        //d (non-existing label)
        {
            let strace = vec!["d".to_string()];
            let trace = slpn.get_activity_key_mut().process_trace(&strace);
            let trace_follower = FollowerSemantics::Trace(&trace);
            let probability = slpn.get_probability(&trace_follower).unwrap();
            assert_eq!(probability, "0".parse::<Fraction>().unwrap());
        }
    }

    #[test]
    fn uemsc_activity_key() {
        let fin1 = fs::read_to_string("testfiles/aa-ab-ba.slang").unwrap();
        let mut slang1 = fin1.parse::<FiniteStochasticLanguage>().unwrap();

        let fin2 = fs::read_to_string("testfiles/ba-aa-ab.slang").unwrap();
        let mut slang2 = fin2.parse::<FiniteStochasticLanguage>().unwrap();

        let uemsc = uemsc(Box::new(slang1), Box::new(slang2)).unwrap();
        assert_eq!(uemsc, Fraction::one())
    }

    #[test]
    fn align_sdfa_trace() {
        let fin = fs::read_to_string("testfiles/aa-ab-ba.sdfa").unwrap();
        let mut sdfa = Rc::new(fin.parse::<StochasticDeterministicFiniteAutomaton>().unwrap());
        let mut semantics = StochasticDeterministicFiniteAutomaton::get_semantics(sdfa);

        let a = semantics.get_activity_key_mut().process_activity("a");
        let b = semantics.get_activity_key_mut().process_activity("b");

        let trace = vec![b, b];
        
        let (alignment, cost) = semantics.align_trace(&trace).unwrap();

        let correct_1 = vec![Move::SynchronousMove(b, 1), Move::ModelMove(a, 4), Move::SilentMove(6), Move::LogMove(b)];
        let correct_2 = vec![Move::SynchronousMove(b, 1), Move::LogMove(b), Move::ModelMove(a, 4), Move::SilentMove(6)];

        assert_eq!(cost, 20001);
        assert!(alignment == correct_1 || alignment == correct_2);
    }

    #[test]
    fn align_sdfa_lang() {
        let fin1 = fs::read_to_string("testfiles/aa-ab-ba.sdfa").unwrap();
        let mut sdfa = Rc::new(fin1.parse::<StochasticDeterministicFiniteAutomaton>().unwrap());
        let mut semantics = StochasticDeterministicFiniteAutomaton::get_semantics(sdfa);

        let fin2 = fs::read_to_string("testfiles/bb.lang").unwrap();
        let mut lang = Box::new(fin2.parse::<FiniteLanguage>().unwrap());

        let a = semantics.get_activity_key_mut().process_activity("a");
        let b = semantics.get_activity_key_mut().process_activity("b");
        
        let alignment = semantics.align_language(lang).unwrap();

        let correct_1 = vec![Move::SynchronousMove(b, 1), Move::ModelMove(a, 4), Move::SilentMove(6), Move::LogMove(b)];
        let correct_2 = vec![Move::SynchronousMove(b, 1), Move::LogMove(b), Move::ModelMove(a, 4), Move::SilentMove(6)];

        assert!(alignment.get(0) == Some(&correct_1) || alignment.get(0) == Some(&correct_2));
    }

    #[test]
    fn align_miner_non_appearing() {
        let fin1 = fs::read_to_string("testfiles/aa-ab-ba.lpn").unwrap();
    }
}