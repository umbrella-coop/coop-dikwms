# Generated from: docs/specs/SPEC-002-moderation-promotion.md
# Generator: /derive bdd v2.0.0
# Generated at: 2026-08-06
# AC Coverage: AC-1..AC-6 (1:1 mapping)

Feature: Moderation & Promotion Workflow
  [Source] From SPEC-002 Overview
  Any change to a scope's property set flows through a change request
  (submitted → approved/rejected), and a child scope's version may be
  promoted to its parent scope. Applied promotions record provenance.

  @SPEC-002 @AC-1
  Scenario: Submitting a change request does not change current truth
    # [Source] From SPEC-002 AC-1
    Given an entity with a current org-level property set version 1
    When a change request proposing org version 2 is submitted
    Then a change request with status submitted is created
    And the org-level set remains version 1

  @SPEC-002 @AC-2
  Scenario: Approved request becomes current truth
    # [Source] From SPEC-002 AC-2
    Given an approved change request proposing org version 2
    When the request is applied
    Then the org-level property set becomes version 2 with status current
    And version 1 remains retrievable as history

  @SPEC-002 @AC-3
  Scenario: Rejected request leaves truth untouched
    # [Source] From SPEC-002 AC-3
    Given a rejected change request proposing workspace version 3
    When the rejection is finalized
    Then the workspace-level property set is unchanged
    And the request status is rejected

  @SPEC-002 @AC-4
  Scenario: Double-apply is a no-op
    # [Source] From SPEC-002 AC-4
    Given an already-applied change request
    When apply is invoked again
    Then no property set changes
    And the outcome reports the request was already applied

  @SPEC-002 @AC-5
  Scenario: Workspace version promotes to org
    # [Source] From SPEC-002 AC-5
    Given a workspace-level property set version 3
    When a change request promoting workspace version 3 to org scope is approved and applied
    Then the org-level property set contains the workspace version 3 properties
    And its provenance records from workspace version 3

  @SPEC-002 @AC-6
  Scenario: Provenance of a promoted set is retrievable
    # [Source] From SPEC-002 AC-6
    Given a promoted org-level property set
    When its provenance is queried
    Then source scope, source version, request id, reviewer, and applied-at time are returned
